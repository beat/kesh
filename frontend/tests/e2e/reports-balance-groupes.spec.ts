/**
 * Story 46-6b — la balance des comptes par groupes, de bout en bout : écran
 * groupé par défaut (quatre colonnes, résultat reporté après les groupes),
 * sous-totaux en pied, niveau 1, ouverture d'un groupe à la main, export CSV et
 * PDF aux mêmes paramètres que l'écran — et leur contenu —, hiérarchie masquée
 * qui rend l'écran d'avant, zéro violation axe sur l'onglet groupé.
 *
 * ⚠️ Ce que seul un E2E vérifie : que les paramètres de groupement traversent la
 * frontière HTTP dans la requête de la balance ET dans celle de son export, et
 * que le fichier téléchargé est groupé comme l'écran.
 *
 * Plan : le preset `with-company`, plus l'arbre `19` > `190` > `1900`, `1910`
 * créé ici, mouvementé contre `3000`.
 *
 * Captures : `KESH_CAPTURES_DIR` (thème clair), une par état, et les deux
 * fichiers exportés.
 */

import { readFileSync, writeFileSync } from 'node:fs';
import AxeBuilder from '@axe-core/playwright';
import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { todayInZurich } from './helpers/api-fixtures';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

const CAPTURES = process.env.KESH_CAPTURES_DIR ?? 'test-results';

async function capture(page: Page, nom: string): Promise<void> {
	await page.screenshot({ path: `${CAPTURES}/46-6b-${nom}.png`, fullPage: true });
}

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function creerCompte(
	api: APIRequestContext,
	data: { number: string; name: string; parentId?: number; postable?: boolean },
): Promise<number> {
	const res = await api.post('/api/v1/accounts', { data: { accountType: 'Asset', ...data } });
	expect(res.ok(), `compte ${data.number} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return (await res.json()).id as number;
}

/** L'arbre 19 > 190 > {1900, 1910}, mouvementé contre 3000 ; rend l'id de 19. */
async function preparerArbre(page: Page): Promise<number> {
	const api = await authedApiContext(page);
	try {
		const g19 = await creerCompte(api, { number: '19', name: 'Actifs du test', postable: false });
		const g190 = await creerCompte(api, {
			number: '190',
			name: 'Liquidités du test',
			parentId: g19,
			postable: false,
		});
		const banque = await creerCompte(api, { number: '1900', name: 'Banque du test', parentId: g190 });
		const caisse = await creerCompte(api, { number: '1910', name: 'Caisse du test', parentId: g190 });
		const acc = await api.get('/api/v1/accounts?includeArchived=false');
		const ventes = ((await acc.json()) as Array<{ id: number; number: string }>).find(
			(a) => a.number === '3000',
		);
		expect(ventes, 'compte 3000 du preset').toBeTruthy();
		const entry = await api.post('/api/v1/journal-entries', {
			data: {
				entryDate: todayInZurich(),
				journal: 'OD',
				description: 'Ventes du test de la balance par groupes',
				lines: [
					{ accountId: banque, debit: '1000.00', credit: '0.00' },
					{ accountId: caisse, debit: '250.00', credit: '0.00' },
					{ accountId: ventes!.id, debit: '0.00', credit: '1250.00' },
				],
			},
		});
		expect(entry.ok(), `écriture : ${entry.status()} ${await entry.text()}`).toBeTruthy();
		return g19;
	} finally {
		await disposeContextSafe(api);
	}
}

const BALANCE = /\/api\/v1\/reports\/trial-balance\?/;

/** Les cellules d'une ligne, espaces resserrés. */
async function cellules(page: Page, testId: string): Promise<string[]> {
	const textes = await page.getByTestId(testId).locator('td').allInnerTexts();
	return textes.map((t) => t.replace(/\s+/g, ' ').trim());
}

/** Exporte l'onglet affiché ; rend l'URL de l'export et le fichier, gardé dans les captures. */
async function exporter(page: Page, format: 'csv' | 'pdf', nom: string): Promise<[URL, Buffer]> {
	const exportReq = page.waitForRequest((r) => /reports\/trial-balance\/export\?/.test(r.url()));
	const download = page.waitForEvent('download');
	await page.getByRole('button', { name: format === 'csv' ? /export csv/i : /export pdf/i }).click();
	const url = new URL((await exportReq).url());
	const fichier = await download;
	const chemin = await fichier.path();
	expect(chemin, 'fichier téléchargé').toBeTruthy();
	const corps = readFileSync(chemin!);
	writeFileSync(`${CAPTURES}/46-6b-${nom}.${format}`, corps);
	return [url, corps];
}

test('balance par groupes : défaut, pied, niveau 1, ouverture à la main, exports, à plat (Story 46-6b)', async ({
	page,
}) => {
	await login(page);
	const g19 = await preparerArbre(page);

	await page.goto('/reports');
	await page.waitForLoadState('networkidle');
	await page.locator('#reports-tab-trial-balance').click();
	await expect(page.getByTestId('grouping-controls')).toBeVisible();

	// 1. Première visite : groupée, tous les comptes, sous-totaux sur le titre (AC-1).
	let req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByRole('button', { name: /générer/i }).click();
	let url = new URL((await req).url());
	expect(url.searchParams.get('detailLevel')).toBe('all');
	expect(url.searchParams.get('subtotalPosition')).toBe('header');
	expect(url.searchParams.has('open')).toBe(false);
	// Ouverture, débit, crédit, clôture du groupe : ses deux comptes.
	expect((await cellules(page, 'grouped-group-19')).slice(2)).toEqual([
		'0.00',
		'1’250.00',
		'0.00',
		'1’250.00',
	]);
	expect((await cellules(page, 'grouped-account-1900')).slice(2)).toEqual([
		'0.00',
		'1’000.00',
		'0.00',
		'1’000.00',
	]);
	await expect(page.getByTestId('group-toggle-19')).toHaveAttribute('aria-expanded', 'true');
	// Le résultat reporté suit les groupes, hors d'eux ; totaux et contrôles restent.
	await expect(page.locator('tbody tr').last()).toHaveAttribute('data-testid', 'tb-retained');
	await expect(page.getByTestId('tb-opening-check')).toHaveText('✓');
	await expect(page.getByTestId('tb-movements-check')).toHaveText('✓');
	await expect(page.getByRole('tabpanel').getByRole('columnheader')).toHaveCount(6);
	await capture(page, '1-defaut-tous-les-comptes');

	// Zéro violation axe sur l'onglet Balance groupé (AC-1).
	const axe = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
	expect(axe.violations).toEqual([]);

	// 2. Sous-totaux en pied : « Total <nom> », le titre sans montant.
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('grouping-position').selectOption('footer');
	expect(new URL((await req).url()).searchParams.get('subtotalPosition')).toBe('footer');
	await expect(page.getByTestId('grouped-subtotal-19')).toContainText('Total Actifs du test');
	expect((await cellules(page, 'grouped-subtotal-19')).slice(2)).toEqual([
		'0.00',
		'1’250.00',
		'0.00',
		'1’250.00',
	]);
	expect((await cellules(page, 'grouped-group-19')).slice(2)).toEqual(['', '', '', '']);
	await capture(page, '2-sous-totaux-en-pied');
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('grouping-position').selectOption('header');
	await req;

	// 3. Niveau 1 : 19 fermé sur son sous-total, sa descendance cachée.
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('grouping-level').selectOption('1');
	expect(new URL((await req).url()).searchParams.get('detailLevel')).toBe('1');
	await expect(page.getByTestId('grouped-account-1900')).toHaveCount(0);
	await expect(page.getByTestId('group-toggle-19')).toHaveAttribute('aria-expanded', 'false');
	await expect(page.getByTestId('grouped-group-19')).toContainText('1’250.00');
	await capture(page, '3-niveau-1');

	// 4. Ouverture de 19 à la main : 190 paraît, fermé par le niveau.
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('group-toggle-19').click();
	url = new URL((await req).url());
	expect(url.searchParams.get('open')).toBe(String(g19));
	await expect(page.getByTestId('group-toggle-190')).toHaveAttribute('aria-expanded', 'false');
	await expect(page.getByTestId('grouped-group-190')).toContainText('1’250.00');
	await expect(page.getByTestId('grouped-account-1900')).toHaveCount(0);
	await capture(page, '4-groupe-ouvert-a-la-main');

	// 5. Export CSV : les paramètres de l'écran, et le fichier groupé comme lui (AC-2).
	const [csvUrl, csv] = await exporter(page, 'csv', '5-export-niveau-1-ouvert');
	expect(csvUrl.searchParams.get('format')).toBe('csv');
	expect(csvUrl.searchParams.get('detailLevel')).toBe('1');
	expect(csvUrl.searchParams.get('subtotalPosition')).toBe('header');
	expect(csvUrl.searchParams.get('open')).toBe(String(g19));
	expect(csvUrl.searchParams.has('closed')).toBe(false);
	const lignes = csv.toString('utf8').replace(/^﻿/, '').split('\r\n');
	expect(lignes[0]).toBe(
		'Niveau;TypeLigne;NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture',
	);
	expect(lignes).toContain('1;groupe;19;Actifs du test;0.00;1250.00;0.00;1250.00');
	expect(lignes).toContain('2;groupe;190;Liquidités du test;0.00;1250.00;0.00;1250.00');
	expect(lignes.some((l) => l.includes(';1900;'))).toBe(false);
	expect(lignes.some((l) => l.startsWith(';;Total;'))).toBe(true);

	// 6. Export PDF, mêmes paramètres : l'en-tête dit le niveau et l'ajustement.
	const [pdfUrl, pdf] = await exporter(page, 'pdf', '6-export-niveau-1-ouvert');
	expect(pdfUrl.searchParams.get('detailLevel')).toBe('1');
	expect(pdfUrl.searchParams.get('open')).toBe(String(g19));
	expect(pdf.subarray(0, 5).toString()).toBe('%PDF-');

	// 7. Hiérarchie masquée : l'écran d'avant, requête sans paramètre.
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('grouping-shown').uncheck();
	url = new URL((await req).url());
	expect(url.searchParams.has('detailLevel')).toBe(false);
	await expect(page.getByTestId('grouped-group-19')).toHaveCount(0);
	await expect(page.getByTestId('tb-row').first()).toBeVisible();
	await expect(page.getByRole('link', { name: '1900' })).toBeVisible();
	await expect(page.getByTestId('grouping-level')).toBeDisabled();
	await capture(page, '7-hierarchie-masquee');

	// La remise en place laisse la vue par défaut pour les specs suivantes.
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('grouping-shown').check();
	await req;
	await page.getByTestId('grouping-open-all').click();
});
