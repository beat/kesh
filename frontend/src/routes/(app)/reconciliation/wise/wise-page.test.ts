// Story 39-11d (AC-2) — la page Wise, sur la page commune des prestataires : aucun compte,
// consigne d'export à l'écran, choix multiple (zip ou CSV), compte rendu par fichier
// (archive, doublons, soldes, avertissement de continuité, entrées du zip non lues), refus
// traduits (frais non séparés, zip illisible, deux fichiers d'une devise), et, faute de la
// revue Wise (39-11c), ni exercice ni propositions.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
import type { WiseImportResponse } from '$lib/features/wise/wise.types';

vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

const listBankAccountsMock = vi.fn<(archived?: boolean, closed?: boolean) => Promise<BankAccountSummary[]>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: (archived?: boolean, closed?: boolean) => listBankAccountsMock(archived, closed),
}));
const listFiscalYearsMock = vi.fn(async () => [
	{ id: 5, companyId: 1, name: '2020', startDate: '2020-01-01', endDate: '2020-12-31', status: 'Open', multiCurrency: true },
]);
vi.mock('$lib/features/fiscal-years/fiscal-years.api', () => ({ listFiscalYears: () => listFiscalYearsMock() }));
vi.mock('$lib/features/reconciliation/reconciliation.api', () => ({
	getLinkCandidates: vi.fn(async () => ({ bankTransactionId: 0, candidates: [] })),
	getEntryCandidates: vi.fn(async () => ({ bankTransactionId: 0, candidates: [] })),
	linkEntry: vi.fn(),
	linkTransaction: vi.fn(),
}));

const importMock = vi.fn();
vi.mock('$lib/features/wise/wise.api', () => ({
	importWiseFiles: (...a: unknown[]) => importMock(...a),
}));

import Page from './+page.svelte';

function wiseAccount(id: number, account: string, journalAccountId = 31): BankAccountSummary {
	return {
		id,
		bankName: 'Wise',
		description: null,
		iban: null,
		qrIban: null,
		accountKind: 'bank',
		cardLast4: null,
		provider: 'wise',
		paypal: null,
		wise: { account, feeAccountId: 60 },
		stripe: null,
		isPrimary: false,
		journalAccountId,
		version: 1,
		archived: false,
		closedOn: null,
		currentBalance: null,
		lastTransactionDate: null,
		statementClosingBalance: null,
		statementDate: null,
		ledgerBalanceAtStatement: null,
		statementGap: null,
	};
}

function report(): WiseImportResponse {
	return {
		account: 'Wise Fictif',
		files: [
			{
				filename: 'wise-releve-chf-fictif.csv',
				archive: 'statement_csv.zip',
				currency: 'CHF',
				bankAccountId: 1,
				importId: 7,
				rows: 10,
				inserted: 10,
				duplicates: 0,
				openingBalance: '0.00',
				closingBalance: '1234.50',
				warnings: [],
			},
			{
				filename: 'wise-releve-eur-fictif.csv',
				archive: 'statement_csv.zip',
				currency: 'EUR',
				bankAccountId: 2,
				importId: null,
				rows: 5,
				inserted: 0,
				duplicates: 5,
				openingBalance: '10.00',
				closingBalance: '20.00',
				warnings: [{ code: 'gap', expected: '0.00', found: '10.00', previousImportId: 3 }],
			},
		],
		ignoredEntries: [{ archive: 'statement_csv.zip', entry: 'lisez-moi.pdf' }],
	};
}

async function choose(r: { findByTestId: (id: string) => Promise<HTMLElement> }, files: File[]) {
	const input = (await r.findByTestId('wise-file-input')) as HTMLInputElement;
	await fireEvent.change(input, { target: { files } });
	return input;
}

describe('page Wise (Story 39-11d)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		listBankAccountsMock.mockResolvedValue([wiseAccount(1, 'Wise Fictif'), wiseAccount(2, 'Wise Fictif', 32)]);
	});

	it('sans compte Wise : le message et le lien vers les comptes bancaires ; un compte Stripe ne compte pas', async () => {
		listBankAccountsMock.mockResolvedValue([
			{ ...wiseAccount(1, 'x'), provider: 'stripe', wise: null, stripe: { account: 'S', feeAccountId: null, payoutBankAccountId: null } },
		]);
		const r = render(Page);
		expect((await r.findByTestId('wise-no-account')).textContent).toContain('Aucun compte Wise');
		expect(r.getByTestId('wise-no-account-link').getAttribute('href')).toBe('/bank-accounts');
	});

	it('la consigne d’export est à l’écran, le choix prend le zip et plusieurs CSV ; ni exercice ni revue', async () => {
		const r = render(Page);
		const input = (await r.findByTestId('wise-file-input')) as HTMLInputElement;
		expect(input.getAttribute('accept')).toBe('.csv,text/csv,.zip,application/zip');
		expect(input.multiple).toBe(true);
		const aide = r.getByTestId('wise-import-help').textContent ?? '';
		expect(aide).toContain('Afficher les transactions dont les frais sont indiqués séparément');
		expect(aide).toContain('zip');
		expect(r.getByTestId('wise-import-button').textContent).toContain('Importer des relevés Wise');
		expect(r.queryByTestId('wise-fiscal-year')).toBeNull();
		expect(r.queryByTestId('wise-proposals')).toBeNull();
		expect(document.body.textContent).not.toMatch(/automatiquement/i);
	});

	it('import : tous les fichiers en un envoi, compte rendu par fichier, avertissement et entrées non lues', async () => {
		importMock.mockResolvedValue(report());
		const r = render(Page);
		const zip = new File(['PK'], 'statement_csv.zip', { type: 'application/zip' });
		const csv = new File(['x'], 'autre.csv', { type: 'text/csv' });
		await choose(r, [zip, csv]);
		await waitFor(() => expect(importMock).toHaveBeenCalledWith([zip, csv], 'Wise Fictif'));
		const chf = await r.findByTestId('wise-import-report-file-CHF');
		expect(chf.textContent).toContain('wise-releve-chf-fictif.csv (archive statement_csv.zip)');
		expect(chf.textContent).toContain('CHF : 10 transaction(s) insérée(s), 0 doublon(s) ; solde de 0.00 à 1234.50 CHF');
		expect(chf.querySelector('[data-testid="wise-import-report-warning-gap"]')).toBeNull();
		const eur = r.getByTestId('wise-import-report-file-EUR');
		expect(eur.textContent).toContain('0 transaction(s) insérée(s), 5 doublon(s)');
		expect(eur.querySelector('[data-testid="wise-import-report-file-all-known"]')).not.toBeNull();
		expect(eur.querySelector('[data-testid="wise-import-report-warning-gap"]')?.textContent).toContain(
			'solde d’ouverture 10.00 EUR, au lieu de 0.00 EUR',
		);
		expect(r.getByTestId('wise-import-report-ignored').textContent).toContain('lisez-moi.pdf');
	});

	it('refus : relevé exporté sans les frais séparés — le fichier, son zip, la consigne et les lignes', async () => {
		importMock.mockRejectedValue({
			code: 'WISE_CSV_FEES_NOT_SEPARATED',
			status: 422,
			message: 'refus du serveur',
			details: { filename: 'chf.csv', archive: 'statement_csv.zip', lines: [3, 7], count: 2 },
		});
		const r = render(Page);
		await choose(r, [new File(['PK'], 'statement_csv.zip')]);
		const texte = (await r.findByTestId('wise-import-error')).textContent ?? '';
		expect(texte).toContain('« chf.csv » (archive « statement_csv.zip »)');
		expect(texte).toContain('en cochant « Afficher les transactions dont les frais sont indiqués séparément »');
		expect(texte).toContain('2 opération(s) à frais sans leur ligne de frais (lignes 3, 7)');
		expect(r.queryByTestId('wise-import-report')).toBeNull();
	});

	it('refus du zip : le message du serveur, tel quel', async () => {
		importMock.mockRejectedValue({
			code: 'WISE_ZIP_INVALID',
			status: 422,
			message: 'L’archive « x.zip » ne contient aucun relevé CSV : exportez de Wise le relevé au format CSV.',
			details: { filename: 'x.zip', reason: 'NO_CSV' },
		});
		const r = render(Page);
		await choose(r, [new File(['PK'], 'x.zip')]);
		expect((await r.findByTestId('wise-import-error')).textContent).toContain('ne contient aucun relevé CSV');
	});

	// Revue P1, L7 — la page sans revue ne lit pas les exercices : leur lecture en échec ne
	// masque pas l'import.
	it('les exercices ne sont pas lus : leur échec ne masque pas l’import', async () => {
		listFiscalYearsMock.mockRejectedValueOnce(new Error('exercices indisponibles'));
		const r = render(Page);
		expect(await r.findByTestId('wise-file-input')).toBeTruthy();
		expect(r.queryByTestId('wise-load-error')).toBeNull();
		expect(listFiscalYearsMock).not.toHaveBeenCalled();
		listFiscalYearsMock.mockReset();
	});

	it('refus : relevé déjà importé, le nom sous lequel il l’a été', async () => {
		importMock.mockRejectedValue({
			code: 'BANK_IMPORT_DUPLICATE_FILE',
			status: 422,
			message: 'Ce fichier a déjà été importé.',
			details: { existingImportId: 9, existingFilename: 'statement_1_CHF_2020-01-01_2020-12-31.csv' },
		});
		const r = render(Page);
		await choose(r, [new File(['PK'], 'statement_csv.zip')]);
		expect((await r.findByTestId('wise-import-error')).textContent).toContain(
			'sous le nom « statement_1_CHF_2020-01-01_2020-12-31.csv » : rien n’a été importé',
		);
		expect(r.queryByTestId('wise-import-report')).toBeNull();
	});

	it('refus : deux fichiers d’une même devise, nommés', async () => {
		importMock.mockRejectedValue({
			code: 'WISE_DUPLICATE_CURRENCY_FILE',
			status: 422,
			message: 'refus',
			details: { currency: 'EUR', filenames: ['a.csv', 'b.csv'] },
		});
		const r = render(Page);
		await choose(r, [new File(['x'], 'a.csv'), new File(['x'], 'b.csv')]);
		expect((await r.findByTestId('wise-import-error')).textContent).toContain('en EUR (a.csv, b.csv) : un seul fichier par devise');
	});
});
