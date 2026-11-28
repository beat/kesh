/**
 * Story 47-4 (AC-5, AC-6 ; NFR-47-4) — les comptes miroirs, du plan comptable
 * aux rapports et à leurs exports.
 *
 * L'exemple chiffré du fichier d'epic, semé par l'API sur le seed `with-company`
 * (qui ne charge aucun plan livré : 1000 et 2000 y existent, les autres comptes
 * se créent ici, tous **sans parent** — un parent deviendrait non imputable). Les
 * miroirs se posent **à l'écran**, au dialogue du plan comptable (47-2) :
 * 1020 → 2100.1, 2270 → 1040.1, 6950.1 → 6900.1. Puis :
 *
 * - bilan, dans la vue par défaut, groupée (Epic 46 ; sans groupe ici, chaque
 *   compte est une ligne `grouped-account-…` de profondeur 1) : aucune cellule
 *   de montant négative ; 1040.1 à 1’200.00 avec « Solde de 2270 … », 2100.1 à
 *   800.00, totaux 13’700.00 et 3’800.00 ; puis la même mention dans la vue à
 *   plat (hiérarchie masquée), remise ensuite en place ;
 * - compte de résultat : 6900.1 à 250.00 avec « Solde de 6950.1 … », 6950.1
 *   absent des produits, totaux 2’000.00 et 300.00, résultat 1’700.00 ;
 * - balance, vue groupée par défaut : les soldes réels (1020 à -800.00, 2270 à
 *   -1’200.00, 6950.1 à -250.00) ;
 * - exports du bilan (boutons *Export PDF*, *Export CSV*) : 1040.1 et 2100.1 avec
 *   leurs montants présentés, dans la forme propre de chaque export (PDF
 *   `1'200.00`, apostrophe ASCII ; CSV `1200.00`), et ni 1020 ni 2270, cherchés
 *   comme jetons entiers.
 *
 * Aucun libellé de compte du seed n'est asserté (1000, 2000, 1100, 3000 et 4000
 * prennent leur nom du plan PME, pour les figures du manuel). La spec reseede `with-company`
 * à la fin (les miroirs ne doivent pas fuir dans les specs suivantes).
 */

import * as fs from 'node:fs';
import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { todayInZurich } from './helpers/api-fixtures';
import { contenuDuPdf, tjDe } from './helpers/pdf';

test.describe.configure({ mode: 'serial' });

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

const captures = process.env.KESH_CAPTURES_DIR ?? 'test-results';

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function createAccount(
	ctx: APIRequestContext,
	data: Record<string, unknown>,
): Promise<number> {
	const res = await ctx.post('/api/v1/accounts', { data });
	expect(res.ok(), `compte ${data.number} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return ((await res.json()) as { id: number }).id;
}

async function accountId(ctx: APIRequestContext, number: string): Promise<number> {
	const res = await ctx.get('/api/v1/accounts');
	const found = ((await res.json()) as { id: number; number: string }[]).find(
		(a) => a.number === number,
	);
	expect(found, `compte ${number}`).toBeTruthy();
	return found!.id;
}

/** Renomme un compte par l'API (`PUT`, le reste tel qu'en base). */
async function renommer(ctx: APIRequestContext, id: number, name: string): Promise<void> {
	const list = (await (await ctx.get('/api/v1/accounts')).json()) as {
		id: number;
		accountType: string;
		role: string | null;
		postable: boolean;
		version: number;
	}[];
	const a = list.find((x) => x.id === id)!;
	const res = await ctx.put(`/api/v1/accounts/${id}`, {
		data: { name, accountType: a.accountType, role: a.role, postable: a.postable, version: a.version },
	});
	expect(res.ok(), `renommage de ${id} : ${res.status()} ${await res.text()}`).toBeTruthy();
}

/** Pose à l'écran le compte miroir d'une source (dialogue du plan comptable, 47-2). */
async function poserMiroir(page: Page, source: string, miroirId: number): Promise<void> {
	await page.getByTestId(`account-row-${source}-edit-button`).click();
	await page.getByTestId('account-edit-dialog-mirror').selectOption(String(miroirId));
	await page.getByTestId('account-edit-dialog-submit').click();
	await expect(page.getByTestId('account-edit-dialog-submit')).toHaveCount(0);
}

/**
 * Aucune cellule de montant d'une ligne de compte du rapport ne commence par `-`,
 * dans la vue groupée (`grouped-account-…`) comme dans la vue à plat (`report-row-…`).
 */
async function aucunMontantNegatif(page: Page): Promise<void> {
	const cellules = page.locator(
		'[data-testid^="grouped-account-"] td.text-right, [data-testid^="report-row-"] td.text-right',
	);
	const n = await cellules.count();
	expect(n, 'cellules de montant des lignes de compte').toBeGreaterThan(0);
	for (const texte of await cellules.allTextContents()) {
		expect(texte.trim().startsWith('-'), `montant négatif « ${texte} »`).toBe(false);
	}
}

/**
 * La ligne d'un compte dans la vue groupée, celle de la première visite (46-2, Q10) :
 * bilan, compte de résultat et balance.
 */
function ligne(page: Page, numero: string) {
	return page.getByTestId(`grouped-account-${numero}`);
}

/** La cellule de montant (dernière) de la ligne groupée d'un compte. */
function montant(page: Page, numero: string) {
	return ligne(page, numero).locator('td').last();
}

/** Télécharge l'export du rapport affiché par son bouton, et rend ses octets. */
async function telecharger(page: Page, bouton: RegExp, chemin: string): Promise<Buffer> {
	const btn = page.getByRole('button', { name: bouton });
	await expect(btn).toBeEnabled({ timeout: 10000 });
	const attente = page.waitForEvent('download');
	await btn.click();
	const download = await attente;
	await download.saveAs(chemin);
	const octets = fs.readFileSync(chemin);
	fs.unlinkSync(chemin);
	return octets;
}

test('l’exemple chiffré : miroirs posés à l’écran, bilan et compte de résultat présentés, balance réelle, exports', async ({
	page,
}) => {
	await login(page);
	const today = todayInZurich();
	const ctx = await authedApiContext(page);
	const ids: Record<string, number> = {};
	try {
		// Comptes de l'exemple (plan PME), sans parent ; les miroirs non imputables.
		for (const [number, name, accountType, extra] of [
			['1010', 'Poste', 'Asset', {}],
			['1020', 'Banque', 'Asset', {}],
			['2270', 'AVS/AI/APG', 'Liability', {}],
			['2800', 'Capital social', 'Liability', { role: 'EquityCapital' }],
			['3200', 'Prestations de services', 'Revenue', {}],
			['6900', 'Charges financières', 'Expense', {}],
			['6950.1', 'Gains de change', 'Revenue', {}],
			['6900.1', 'Pertes de change', 'Expense', { postable: false }],
			['1040.1', 'Créances AVS/AI/APG', 'Asset', { postable: false }],
			['2100.1', 'Découvert bancaire', 'Liability', { postable: false }],
		] as const) {
			ids[number] = await createAccount(ctx, { number, name, accountType, ...extra });
		}
		ids['1000'] = await accountId(ctx, '1000');
		ids['2000'] = await accountId(ctx, '2000');
		// Les comptes du seed prennent leur nom du plan PME, pour les figures du manuel
		// (« une société tient le plan PME livré » ; aucune assertion ne porte sur eux) :
		// 1000 et 2000 de l'exemple, et 1100, 3000, 4000, que la balance montre à zéro.
		for (const [numero, nom] of [
			['1000', 'Caisse'],
			['2000', 'Créanciers'],
			['1100', 'Débiteurs'],
			['3000', 'Ventes de marchandises'],
			['4000', 'Achats de marchandises'],
		] as const) {
			await renommer(ctx, await accountId(ctx, numero), nom);
		}

		// Écritures, chacune contre 2800 (débit + / crédit −).
		const ecritures: [string, string, string][] = [
			['1000', '500.00', '0'],
			['1010', '12000.00', '0'],
			['1020', '0', '800.00'],
			['2000', '0', '3000.00'],
			['2270', '1200.00', '0'],
			['3200', '0', '2000.00'],
			['6900', '50.00', '0'],
			['6950.1', '250.00', '0'],
		];
		for (const [numero, debit, credit] of ecritures) {
			const res = await ctx.post('/api/v1/journal-entries', {
				data: {
					entryDate: today,
					journal: 'OD',
					description: `E2E 47-4 — ${numero}`,
					lines: [
						{ accountId: ids[numero], debit, credit },
						{ accountId: ids['2800'], debit: credit, credit: debit },
					],
				},
			});
			expect(res.ok(), `écriture ${numero} : ${res.status()} ${await res.text()}`).toBeTruthy();
		}
	} finally {
		await disposeContextSafe(ctx);
	}

	// Les miroirs, posés à l'écran.
	await page.goto('/accounts');
	await expect(page).toHaveURL('/accounts');
	await poserMiroir(page, '1020', ids['2100.1']);
	await poserMiroir(page, '2270', ids['1040.1']);
	await poserMiroir(page, '6950.1', ids['6900.1']);

	// Bilan présenté.
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');
	// Story 55-1a (D16) — plus de *Générer* : le bilan se charge à l'ouverture de la page, l'onglet le recharge.
	await page.locator('#reports-tab-balance-sheet').click();
	await expect(ligne(page, '1040.1')).toBeVisible({ timeout: 5000 });
	await aucunMontantNegatif(page);
	await expect(montant(page, '1040.1')).toHaveText('1’200.00');
	await expect(ligne(page, '1040.1').getByTestId('mirror-of-1040.1')).toHaveText(
		'Solde de 2270 AVS/AI/APG',
	);
	await expect(montant(page, '2100.1')).toHaveText('800.00');
	await expect(ligne(page, '2100.1').getByTestId('mirror-of-2100.1')).toHaveText(
		'Solde de 1020 Banque',
	);
	await expect(ligne(page, '1020')).toHaveCount(0);
	await expect(ligne(page, '2270')).toHaveCount(0);
	const tfoot = page.locator('tfoot');
	await expect(tfoot.filter({ hasText: /total actifs/i })).toContainText('13’700.00');
	await expect(tfoot.filter({ hasText: /total passifs/i })).toContainText('3’800.00');
	await page.screenshot({ path: `${captures}/mi-bilan.png`, fullPage: true });

	// Exports du bilan : montants présentés, sources absentes (jetons entiers).
	const pdf = contenuDuPdf(await telecharger(page, /export pdf/i, '/tmp/kesh-e2e-47-4-bilan.pdf'));
	expect(pdf).toContain(tjDe('1040.1'));
	expect(pdf).toContain(tjDe("1'200.00"));
	expect(pdf).toContain(tjDe('2100.1'));
	expect(pdf).toContain(tjDe('800.00'));
	expect(pdf).toContain(tjDe("13'700.00"));
	expect(pdf).not.toContain(tjDe('1020'));
	expect(pdf).not.toContain(tjDe('2270'));
	expect(pdf).not.toContain(tjDe('-800.00'));
	expect(pdf).not.toContain(tjDe("-1'200.00"));

	const csv = (await telecharger(page, /export csv/i, '/tmp/kesh-e2e-47-4-bilan.csv'))
		.toString('utf8')
		.replace(/^﻿/, '');
	const champs = csv.split(/\r?\n/).map((l) => l.split(';').map((c) => c.replace(/^"|"$/g, '')));
	const ligneCsv = (numero: string) => champs.find((c) => c.includes(numero));
	expect(ligneCsv('1040.1'), 'ligne 1040.1 du CSV').toContain('1200.00');
	expect(ligneCsv('2100.1'), 'ligne 2100.1 du CSV').toContain('800.00');
	expect(ligneCsv('1020'), 'aucune ligne 1020 au CSV').toBeUndefined();
	expect(ligneCsv('2270'), 'aucune ligne 2270 au CSV').toBeUndefined();
	expect(champs.flat()).not.toContain('-800.00');
	expect(champs.flat()).not.toContain('-1200.00');

	// Vue à plat (hiérarchie masquée) : la même mention ; puis la vue par défaut remise.
	let req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.getByTestId('grouping-shown').uncheck();
	await req;
	await expect(page.getByTestId('report-row-1040.1')).toBeVisible({ timeout: 5000 });
	await aucunMontantNegatif(page);
	await expect(page.getByTestId('report-row-1040.1').locator('td').last()).toHaveText('1’200.00');
	await expect(
		page.getByTestId('report-row-1040.1').getByTestId('mirror-of-1040.1'),
	).toHaveText('Solde de 2270 AVS/AI/APG');
	await expect(
		page.getByTestId('report-row-2100.1').getByTestId('mirror-of-2100.1'),
	).toHaveText('Solde de 1020 Banque');
	await expect(page.getByTestId('report-row-1020')).toHaveCount(0);
	req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.getByTestId('grouping-shown').check();
	await req;
	await expect(ligne(page, '1040.1')).toBeVisible({ timeout: 5000 });

	// Compte de résultat présenté.
	// Story 55-1a (D16) — le clic sur l'onglet charge le compte de résultat, sans *Générer*.
	await page.locator('#reports-tab-income-statement').click();
	await expect(ligne(page, '6900.1')).toBeVisible({ timeout: 5000 });
	await aucunMontantNegatif(page);
	await expect(montant(page, '6900.1')).toHaveText('250.00');
	await expect(ligne(page, '6900.1').getByTestId('mirror-of-6900.1')).toHaveText(
		'Solde de 6950.1 Gains de change',
	);
	await expect(ligne(page, '6950.1')).toHaveCount(0);
	const pied = page.locator('tfoot');
	await expect(pied.filter({ hasText: /total produits/i })).toContainText('2’000.00');
	await expect(pied.filter({ hasText: /total charges/i })).toContainText('300.00');
	await expect(page.locator('p', { hasText: /résultat net/i })).toContainText('1’700.00');
	await page.screenshot({ path: `${captures}/mi-resultat.png`, fullPage: true });

	// Balance de vérification : les soldes réels.
	// Story 55-1a (D16) — le clic sur l'onglet charge la balance, sans *Générer*.
	await page.locator('#reports-tab-trial-balance').click();
	// Vue groupée : la dernière colonne est la clôture ; aucune mention de miroir.
	await expect(ligne(page, '1020')).toBeVisible({ timeout: 5000 });
	await expect(montant(page, '1020')).toHaveText('-800.00');
	await expect(montant(page, '2270')).toHaveText('-1’200.00');
	await expect(montant(page, '6950.1')).toHaveText('-250.00');
	await expect(page.locator('[data-testid^="mirror-of-"]')).toHaveCount(0);
	await page.screenshot({ path: `${captures}/mi-balance.png`, fullPage: true });
});
