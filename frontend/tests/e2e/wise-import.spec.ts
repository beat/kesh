/**
 * Story 39-11d (AC-5) — l'écran Wise, de l'écran au serveur.
 *
 * Société `with-company` (exercice ouvert au 01.01.2020) passée en multi-devises
 * par l'API (comptes de change désignés), comptes du plan CHF, EUR et USD des
 * soldes Wise, compte de frais en CHF.
 *
 * 1. Avant tout compte Wise, la page le dit. Le compte Wise CHF se crée **par
 *    l'écran** (case « Compte Wise », compte de frais) ; EUR et USD par l'API,
 *    même libellé. La liste dit « Wise · <compte> · <devise> ».
 * 2. La consigne d'export est à l'écran. Un relevé CHF exporté **sans** les frais
 *    séparés (la fixture CHF sans ses lignes `FEE-`, montants frais compris :
 *    le patron de `chf_sans_frais` de `wise_import_e2e.rs`) est refusé, la
 *    consigne dite, rien d'importé (relu au serveur : aucun import `chf.csv`).
 * 3. Le **zip** des trois fixtures fictives de 39-11a (lues par chemin, ni copie
 *    ni seconde fixture ; le zip est fait par la spec, avec une entrée PDF que
 *    l'import ignore) s'importe **par l'écran**, d'un geste : compte rendu par
 *    fichier (archive, insérées, soldes), entrée non lue nommée.
 * 4. Le même zip, donné de nouveau : refusé comme déjà importé, le relevé nommé
 *    (`BANK_IMPORT_DUPLICATE_FILE`, son texte propre), aucun import de plus.
 *
 * La revue et l'acceptation Wise (Story 39-11c) ne sont pas dans cette branche :
 * la page n'a que l'import (AC-3 reporté, voir la fiche).
 *
 * L'activation du multi-devises est irréversible : la spec reseede `with-company`
 * en fin de parcours. Captures d'écran (thème clair) dans `KESH_CAPTURES_DIR`.
 */

import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { crc32 } from 'node:zlib';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';

test.describe.configure({ mode: 'serial' });

const CAPTURES = process.env.KESH_CAPTURES_DIR;
const FIXTURES = join(process.cwd(), '../crates/kesh-import/tests/fixtures');
const FIXTURE = (currency: string) => join(FIXTURES, `wise-releve-${currency}-fictif.csv`);
const WISE = 'Wise Fictif';
const ZIP_NAME = 'statement_2020-01-01_2020-12-31_csv.zip';

async function capture(page: Page, name: string): Promise<void> {
	if (CAPTURES) await page.screenshot({ path: join(CAPTURES, `${name}.png`), fullPage: true });
}

async function captureOf(page: Page, testId: string, name: string): Promise<void> {
	if (CAPTURES) await page.getByTestId(testId).screenshot({ path: join(CAPTURES, `${name}.png`) });
}

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function api<T>(ctx: APIRequestContext, method: 'get' | 'post' | 'put', path: string, data?: unknown): Promise<T> {
	const res = await ctx[method](path, data === undefined ? undefined : { data });
	expect(res.ok(), `${method.toUpperCase()} ${path} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return (await res.json()) as T;
}

/**
 * Un zip « stored » (sans compression), de ses entrées dans l'ordre donné — de quoi
 * reproduire l'archive que Wise fait télécharger, sans dépendance.
 */
function storedZip(entries: Array<{ name: string; data: Buffer }>): Buffer {
	const locals: Buffer[] = [];
	const centrals: Buffer[] = [];
	let offset = 0;
	for (const e of entries) {
		const name = Buffer.from(e.name, 'utf8');
		const crc = crc32(e.data);
		const local = Buffer.alloc(30);
		local.writeUInt32LE(0x04034b50, 0);
		local.writeUInt16LE(20, 4);
		local.writeUInt16LE(0x0800, 6); // noms en UTF-8
		local.writeUInt16LE(0, 8); // stored
		local.writeUInt32LE(crc, 14);
		local.writeUInt32LE(e.data.length, 18);
		local.writeUInt32LE(e.data.length, 22);
		local.writeUInt16LE(name.length, 26);
		locals.push(local, name, e.data);
		const central = Buffer.alloc(46);
		central.writeUInt32LE(0x02014b50, 0);
		central.writeUInt16LE(20, 4);
		central.writeUInt16LE(20, 6);
		central.writeUInt16LE(0x0800, 8);
		central.writeUInt32LE(crc, 16);
		central.writeUInt32LE(e.data.length, 20);
		central.writeUInt32LE(e.data.length, 24);
		central.writeUInt16LE(name.length, 28);
		central.writeUInt32LE(offset, 42);
		centrals.push(central, name);
		offset += local.length + name.length + e.data.length;
	}
	const centralDir = Buffer.concat(centrals);
	const end = Buffer.alloc(22);
	end.writeUInt32LE(0x06054b50, 0);
	end.writeUInt16LE(entries.length, 8);
	end.writeUInt16LE(entries.length, 10);
	end.writeUInt32LE(centralDir.length, 12);
	end.writeUInt32LE(offset, 16);
	return Buffer.concat([...locals, centralDir, end]);
}

/** Le relevé CHF tel que Wise l'exporte **sans** l'option des frais séparés (patron de `chf_sans_frais`). */
function chfSansFrais(): Buffer {
	let s = readFileSync(FIXTURE('chf'), 'utf8')
		.split('\n')
		.filter((l) => l !== '' && !l.startsWith('FEE-'))
		.map((l) => `${l}\n`)
		.join('');
	for (const [avant, apres] of [
		[',-300.00,CHF,', ',-301.20,CHF,'],
		[',-500.00,CHF,', ',-500.35,CHF,'],
	]) {
		expect(s.split(avant).length - 1, avant).toBe(1);
		s = s.replace(avant, apres);
	}
	return Buffer.from(s, 'utf8');
}

function wiseZip(): Buffer {
	return storedZip([
		{ name: 'statement_1_CHF_2020-01-01_2020-12-31.csv', data: readFileSync(FIXTURE('chf')) },
		{ name: 'statement_1_EUR_2020-01-01_2020-12-31.csv', data: readFileSync(FIXTURE('eur')) },
		{ name: 'statement_1_USD_2020-01-01_2020-12-31.csv', data: readFileSync(FIXTURE('usd')) },
		{ name: 'lisez-moi.pdf', data: Buffer.from('%PDF-1.4 fictif') },
	]);
}

interface Ids {
	chf: number;
	eur: number;
	usd: number;
	fee: number;
}
let ids: Ids;

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterAll(async () => {
	await seedTestState('with-company');
});

test.beforeEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('1 — préparation ; compte Wise CHF par l’écran, EUR et USD par l’API', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/wise');
	await expect(page.getByTestId('wise-no-account')).toBeVisible();
	await expect(page.getByTestId('wise-no-account-link')).toHaveAttribute('href', '/bank-accounts');
	await capture(page, '00-aucun-compte-wise');
	const ctx = await authedApiContext(page);
	try {
		const create = async (number: string, name: string, accountType: string, extra: object = {}) =>
			(await api<{ id: number }>(ctx, 'post', '/api/v1/accounts', { number, name, accountType, ...extra })).id;
		const transit = await create('1090', 'Compte de conversion', 'Asset');
		const gain = await create('6840', 'Gains de change', 'Revenue');
		const loss = await create('6940', 'Pertes de change', 'Expense');
		const company = await api<{ company: { version: number } }>(ctx, 'get', '/api/v1/companies/current');
		await api(ctx, 'put', '/api/v1/companies/current/currency-settings', {
			multiCurrencyEnabled: true,
			fxGainAccountId: gain,
			fxLossAccountId: loss,
			fxTransitAccountId: transit,
			version: company.company.version,
		});
		ids = {
			chf: await create('1071', 'Wise CHF', 'Asset'),
			eur: await create('1072', 'Wise EUR', 'Asset', { currency: 'EUR' }),
			usd: await create('1073', 'Wise USD', 'Asset', { currency: 'USD' }),
			fee: await create('6947', 'Frais Wise', 'Expense'),
		};
	} finally {
		await disposeContextSafe(ctx);
	}

	// Le compte Wise CHF, par l'écran : la case « Compte Wise » exclut les deux autres.
	await page.goto('/bank-accounts');
	await page.getByTestId('create-bank-account-button').click();
	await page.getByTestId('form-bank-name').fill('Wise');
	await page.getByTestId('form-is-paypal').check();
	await page.getByTestId('form-is-wise').check();
	await expect(page.getByTestId('form-is-paypal')).not.toBeChecked();
	await expect(page.getByTestId('form-is-stripe')).not.toBeChecked();
	await expect(page.getByTestId('form-is-wise-help')).toContainText('page Wise');
	await page.getByTestId('form-wise-account').fill(WISE);
	await page.getByTestId('form-journal-account').selectOption(String(ids.chf));
	await page.getByTestId('form-fee-account').selectOption(String(ids.fee));
	await expect(page.getByTestId('form-qr-iban')).toHaveCount(0);
	await expect(page.getByTestId('form-payout-account')).toHaveCount(0);
	await capture(page, '01-formulaire-compte-wise');
	await page.getByTestId('form-submit').click();
	await expect(page.getByTestId('create-bank-account-form')).toHaveCount(0);
	await expect(page.locator('[data-testid^="bank-account-wise-"]').filter({ hasText: `Wise · ${WISE} · CHF` })).toHaveCount(1);

	const ctx2 = await authedApiContext(page);
	try {
		await api(ctx2, 'post', '/api/v1/bank-accounts', {
			bankName: 'Wise EUR',
			journalAccountId: ids.eur,
			wise: { account: WISE, feeAccountId: ids.fee },
		});
		await api(ctx2, 'post', '/api/v1/bank-accounts', {
			bankName: 'Wise USD',
			journalAccountId: ids.usd,
			wise: { account: WISE, feeAccountId: null },
		});
	} finally {
		await disposeContextSafe(ctx2);
	}
	await page.reload();
	await expect(page.locator('[data-testid^="bank-account-wise-"]')).toHaveCount(3);
	await expect(page.locator('[data-testid^="bank-account-wise-"]').filter({ hasText: `Wise · ${WISE} · USD` })).toHaveCount(1);
	await capture(page, '02-liste-comptes-wise');
});

test('2 — consigne d’export ; relevé sans les frais séparés refusé, rien d’importé', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/wise');
	await expect(page.getByTestId('wise-account')).toHaveValue(WISE);
	await expect(page.getByTestId('wise-fiscal-year')).toHaveCount(0);
	await expect(page.getByTestId('wise-import-help')).toContainText(
		'Afficher les transactions dont les frais sont indiqués séparément',
	);
	await expect(page.getByTestId('wise-file-input')).toHaveAttribute('accept', '.csv,text/csv,.zip,application/zip');
	await capture(page, '03-page-wise-consigne');
	await page.getByTestId('wise-file-input').setInputFiles({ name: 'chf.csv', mimeType: 'text/csv', buffer: chfSansFrais() });
	const refus = page.getByTestId('wise-import-error');
	await expect(refus).toContainText('« chf.csv » a été exporté sans les frais séparés');
	await expect(refus).toContainText('en cochant « Afficher les transactions dont les frais sont indiqués séparément »');
	await expect(refus).toContainText('2 opération(s) à frais sans leur ligne de frais (lignes 6, 7)');
	await expect(page.getByTestId('wise-import-report')).toHaveCount(0);
	await captureOf(page, 'wise-import-error', '04-refus-frais-non-separes');
	// Rien d'importé, relu au serveur : aucun import du relevé refusé.
	const ctx = await authedApiContext(page);
	try {
		const imports = await api<{ items: Array<{ filename: string }> }>(ctx, 'get', '/api/v1/bank-imports?limit=100');
		expect(imports.items.map((i) => i.filename)).not.toContain('chf.csv');
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('3 — le zip de Wise, d’un geste : compte rendu par fichier ; donné de nouveau, refusé', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/wise');
	await expect(page.getByTestId('wise-account')).toHaveValue(WISE);
	const zip = wiseZip();
	await page.getByTestId('wise-file-input').setInputFiles({ name: ZIP_NAME, mimeType: 'application/zip', buffer: zip });
	const rapport = page.getByTestId('wise-import-report');
	await expect(rapport).toBeVisible();
	await expect(page.getByTestId('wise-import-error')).toHaveCount(0);
	const chf = page.getByTestId('wise-import-report-file-CHF');
	await expect(chf).toContainText(`statement_1_CHF_2020-01-01_2020-12-31.csv (archive ${ZIP_NAME})`);
	await expect(chf).toContainText('CHF : 10 transaction(s) insérée(s), 0 doublon(s)');
	await expect(page.getByTestId('wise-import-report-file-EUR')).toContainText('EUR : 5 transaction(s) insérée(s), 0 doublon(s)');
	await expect(page.getByTestId('wise-import-report-file-USD')).toContainText('USD : 3 transaction(s) insérée(s), 0 doublon(s)');
	await expect(page.getByTestId('wise-import-report-ignored')).toContainText('lisez-moi.pdf');
	await expect(rapport.locator('[data-testid^="wise-import-report-warning-"]')).toHaveCount(0);
	await captureOf(page, 'wise-import-report', '05-compte-rendu-zip');
	await capture(page, '06-page-wise-apres-import');

	// Le même zip, de nouveau : refusé comme déjà importé (texte propre à BANK_IMPORT_DUPLICATE_FILE,
	// le premier relevé du zip nommé), rien de plus.
	await page.getByTestId('wise-file-input').setInputFiles({ name: ZIP_NAME, mimeType: 'application/zip', buffer: zip });
	await expect(page.getByTestId('wise-import-error-line')).toHaveText(
		'Un relevé de cet envoi a déjà été importé, sous le nom « statement_1_CHF_2020-01-01_2020-12-31.csv » : rien n’a été importé.',
	);
	await expect(page.getByTestId('wise-import-report')).toHaveCount(0);
	await captureOf(page, 'wise-import-error', '07-zip-deja-importe');
	const ctx = await authedApiContext(page);
	try {
		const imports = await api<{ items: Array<{ filename: string }> }>(ctx, 'get', '/api/v1/bank-imports?limit=100');
		expect(imports.items.filter((i) => i.filename.startsWith('statement_1_')).map((i) => i.filename).sort()).toEqual([
			'statement_1_CHF_2020-01-01_2020-12-31.csv',
			'statement_1_EUR_2020-01-01_2020-12-31.csv',
			'statement_1_USD_2020-01-01_2020-12-31.csv',
		]);
	} finally {
		await disposeContextSafe(ctx);
	}
});
