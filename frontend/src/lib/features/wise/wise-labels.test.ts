// Story 39-11d (AC-2) — libellés de l'écran Wise : chaque refus du lecteur (39-11a) se dit par
// sa cause, le fichier et son archive nommés ; le relevé sans les frais séparés (39-11e) porte
// la consigne ; un refus non propre à Wise garde le message du serveur ; l'avertissement de
// continuité et la ligne du compte rendu.

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

import {
	WISE_LINES_SHOWN,
	wiseFileReportLine,
	wiseImportErrorLines,
	wiseLineErrorLabel,
	wiseLinesList,
	wiseWarningLabel,
} from './wise-labels';

const refus = (code: string, details: Record<string, unknown>, message = 'message du serveur') => ({
	code,
	status: 422,
	message,
	details,
});
const invalid = (reason: string, more: Record<string, unknown> = {}) =>
	refus('WISE_CSV_INVALID', { filename: 'chf.csv', reason, ...more });

describe('wiseImportErrorLines (Story 39-11d)', () => {
	it('frais non séparés : le fichier et son zip, la consigne, les lignes bornées', () => {
		const lines = Array.from({ length: 12 }, (_, i) => i + 2);
		const out = wiseImportErrorLines(
			refus('WISE_CSV_FEES_NOT_SEPARATED', { filename: 'chf.csv', archive: 'r.zip', lines, count: 40 }),
		);
		expect(out[0]).toContain('« chf.csv » (archive « r.zip »)');
		expect(out[0]).toContain('exporté sans les frais séparés');
		expect(out[1]).toContain('« Afficher les transactions dont les frais sont indiqués séparément »');
		expect(out[2]).toBe(`40 opération(s) à frais sans leur ligne de frais (lignes ${lines.slice(0, WISE_LINES_SHOWN).join(', ')}…).`);
	});

	it('frais non séparés d’un fichier envoyé à part, sans lignes : deux phrases, sans archive', () => {
		const out = wiseImportErrorLines(refus('WISE_CSV_FEES_NOT_SEPARATED', { filename: 'eur.csv', archive: null }));
		expect(out).toHaveLength(2);
		expect(out[0]).toContain('« eur.csv » a été');
		expect(out[0]).not.toContain('archive');
	});

	it('deux fichiers d’une même devise, nommés', () => {
		expect(wiseImportErrorLines(refus('WISE_DUPLICATE_CURRENCY_FILE', { currency: 'USD', filenames: ['a.csv', 'b.csv'] }))).toEqual([
			'Deux fichiers portent le relevé en USD (a.csv, b.csv) : un seul fichier par devise. Rien n’a été importé.',
		]);
	});

	it.each([
		['WISE_CSV_MISSING_COLUMNS', { columns: ['Running Balance', 'ID'] }, 'colonnes manquantes (Running Balance, ID)'],
		['WISE_CSV_DUPLICATE_COLUMN', { columns: ['Amount'] }, 'colonne en double : Amount'],
		['WISE_CSV_EMPTY', {}, 'est vide'],
		['WISE_CSV_ENCODING', { cause: 'x' }, 'L’encodage du fichier « chf.csv »'],
		['WISE_CSV_MIXED_CURRENCIES', { currencies: ['CHF', 'EUR'] }, 'plusieurs devises (CHF, EUR)'],
		['WISE_CSV_DUPLICATE_ID', { id: 'CARD-9', lines: [4, 9] }, 'deux fois l’identifiant CARD-9 dans le même sens (lignes 4, 9)'],
		['WISE_CSV_RUNNING_BALANCE_BREAK', { line: 6, expected: '10.00', found: '12.00' }, 'à la ligne 6, le solde courant vaut 12.00 au lieu de 10.00'],
	])('relevé refusé %s : la cause, le fichier nommé', (reason, more, attendu) => {
		const out = wiseImportErrorLines(invalid(reason, more));
		expect(out).toHaveLength(1);
		expect(out[0]).toContain(attendu);
		expect(out[0]).toContain('« chf.csv »');
	});

	it('relevé refusé WISE_CSV_LINES : une phrase par ligne, cause traduite', () => {
		const out = wiseImportErrorLines(
			invalid('WISE_CSV_LINES', {
				archive: 'r.zip',
				lines: [
					{ line: 3, field: 'Date', code: 'DATE', value: '2020-13-01' },
					{ line: 5, field: null, code: 'INCONNU', value: null },
				],
			}),
		);
		expect(out[0]).toBe('Lignes en erreur dans le fichier « chf.csv » (archive « r.zip »), rien n’a été importé :');
		expect(out[1]).toBe('Ligne 3, Date : date illisible ou hors format (JJ-MM-AAAA ou AAAA-MM-JJ attendu)');
		expect(out[2]).toBe('Ligne 5, — : INCONNU');
	});

	// Revue P1, M1 — le refus de doublon nomme le relevé déjà importé (le serveur
	// rend `existingFilename`) ; sans nom, le message du serveur.
	it('relevé déjà importé : le nom sous lequel il l’a été', () => {
		expect(
			wiseImportErrorLines(
				refus('BANK_IMPORT_DUPLICATE_FILE', { existingImportId: 12, existingFilename: 'statement_1_CHF.csv' }),
			),
		).toEqual(['Un relevé de cet envoi a déjà été importé, sous le nom « statement_1_CHF.csv » : rien n’a été importé.']);
		expect(wiseImportErrorLines(refus('BANK_IMPORT_DUPLICATE_FILE', { existingImportId: 12 }, 'déjà importé'))).toEqual([
			'déjà importé',
		]);
	});

	it('raison inconnue, zip refusé, autre code, erreur non API : le message du serveur', () => {
		expect(wiseImportErrorLines(invalid('WISE_CSV_NOUVEAU'))).toEqual(['message du serveur']);
		expect(wiseImportErrorLines(refus('WISE_ZIP_INVALID', { filename: 'x.zip', reason: 'ENCRYPTED' }, 'zip chiffré'))).toEqual([
			'zip chiffré',
		]);
		expect(wiseImportErrorLines(refus('PROVIDER_CURRENCY_ACCOUNT_MISSING', { currencies: ['USD'] }, 'USD sans compte'))).toEqual([
			'USD sans compte',
		]);
		expect(wiseImportErrorLines(new Error('réseau'))).toHaveLength(1);
	});
});

describe('libellés du lecteur, de la continuité et du compte rendu (Story 39-11d)', () => {
	it.each([
		['FIELD_COUNT', 'nombre de cellules'],
		['DATE_TIME', 'horodatage'],
		['AMOUNT', 'montant illisible'],
		['RATE', 'cours de change'],
		['CURRENCY', 'devise illisible'],
		['ID', 'identifiant Wise'],
		['DIRECTION', 'CREDIT ou DEBIT'],
		['SIGN_MISMATCH', 'signe du montant'],
		['FEE_SIGN', 'frais au crédit'],
		['OVERFLOW', 'trop grand'],
		['REQUIRED', 'obligatoire vide'],
	])('%s', (code, attendu) => {
		expect(wiseLineErrorLabel({ code })).toContain(attendu);
	});

	it('liste de lignes : bornée, « … » au-delà', () => {
		expect(wiseLinesList([1, 2, 3])).toBe('1, 2, 3');
		expect(wiseLinesList([1, 2, 3], 5)).toBe('1, 2, 3…');
	});

	it('avertissement gap : les deux soldes et la devise ; code inconnu tel quel', () => {
		expect(wiseWarningLabel({ code: 'gap', expected: '5.00', found: '7.00', previousImportId: 1 }, 'EUR')).toContain(
			'solde d’ouverture 7.00 EUR, au lieu de 5.00 EUR',
		);
		expect(wiseWarningLabel({ code: 'autre', expected: '0', found: '0', previousImportId: 1 }, 'EUR')).toBe('autre');
	});

	it('ligne du compte rendu d’un fichier', () => {
		expect(
			wiseFileReportLine({
				filename: 'f.csv',
				archive: null,
				currency: 'USD',
				bankAccountId: 1,
				importId: 2,
				rows: 3,
				inserted: 2,
				duplicates: 1,
				openingBalance: '0.00',
				closingBalance: '9.50',
				warnings: [],
			}),
		).toBe('USD : 2 transaction(s) insérée(s), 1 doublon(s) ; solde de 0.00 à 9.50 USD');
	});
});
