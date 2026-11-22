/**
 * Story 39-11d (AC-2) — libellés traduits de l'écran Wise : refus de l'import
 * (une phrase par cause, le fichier nommé), avertissement de continuité d'un
 * relevé, compte rendu d'un fichier.
 *
 * ⚠️ **Les clés sont écrites en toutes lettres** (jamais construites par
 * gabarit) : `i18n-keys.test.ts` les voit dès qu'elles manquent d'un catalogue.
 *
 * Les refus de l'archive (`WISE_ZIP_INVALID`, sept raisons) et ceux qui ne
 * sont pas propres à Wise (`PROVIDER_*`, `BANK_IMPORT_DUPLICATE_FILE`,
 * `BANK_ACCOUNT_CLOSED`…) se disent par le **message du serveur**, déjà
 * traduit : il nomme l'archive et dit quoi faire (amendement de la 39-11e).
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { isApiError } from '$lib/shared/utils/api-error';
import { errorMessageOf } from '$lib/shared/utils/error-message';
import type { ProviderLineError } from '$lib/features/provider/provider-review.types';
import type { WiseFileReport, WiseFileWarning } from './wise.types';

/** Nombre de numéros de ligne cités au plus par une phrase de refus (le serveur en rend 100). */
export const WISE_LINES_SHOWN = 10;

/** Libellé d'une erreur de ligne du lecteur Wise (39-11a, `WiseLineErrorCode`) ; code brut si inconnu. */
export function wiseLineErrorLabel(l: Pick<ProviderLineError, 'code'>): string {
	switch (l.code) {
		case 'FIELD_COUNT':
			return i18nMsg('wise-csv-line-field-count', 'nombre de cellules différent de l’en-tête');
		case 'DATE':
			return i18nMsg('wise-csv-line-date', 'date illisible ou hors format (JJ-MM-AAAA attendu)');
		case 'DATE_TIME':
			return i18nMsg('wise-csv-line-date-time', 'horodatage illisible');
		case 'AMOUNT':
			return i18nMsg('wise-csv-line-amount', 'montant illisible ou vide (point décimal attendu)');
		case 'RATE':
			return i18nMsg('wise-csv-line-rate', 'cours de change illisible');
		case 'CURRENCY':
			return i18nMsg('wise-csv-line-currency', 'devise illisible (trois lettres attendues)');
		case 'ID':
			return i18nMsg('wise-csv-line-id', 'identifiant Wise vide ou illisible');
		case 'DIRECTION':
			return i18nMsg('wise-csv-line-direction', 'sens autre que CREDIT ou DEBIT');
		case 'SIGN_MISMATCH':
			return i18nMsg('wise-csv-line-sign-mismatch', 'le signe du montant ne suit pas le sens (crédit ou débit)');
		case 'FEE_SIGN':
			return i18nMsg('wise-csv-line-fee-sign', 'une ligne de frais au crédit');
		case 'OVERFLOW':
			return i18nMsg('wise-csv-line-overflow', 'montant trop grand');
		case 'REQUIRED':
			return i18nMsg('wise-csv-line-required', 'cellule obligatoire vide');
		default:
			return l.code;
	}
}

type Details = Record<string, unknown> | undefined;

function stringsOf(value: unknown): string[] {
	return Array.isArray(value) ? value.filter((v): v is string => typeof v === 'string') : [];
}

function numbersOf(value: unknown): number[] {
	return Array.isArray(value) ? value.filter((v): v is number => typeof v === 'number') : [];
}

function lineErrorsOf(details: Details): ProviderLineError[] {
	const lines = details?.lines;
	return Array.isArray(lines) ? (lines as ProviderLineError[]).filter((l) => typeof l === 'object' && l !== null) : [];
}

/** Les numéros de ligne cités, bornés à {@link WISE_LINES_SHOWN}, suivis de « … » au-delà. */
export function wiseLinesList(lines: readonly number[], total = lines.length): string {
	const shown = lines.slice(0, WISE_LINES_SHOWN).join(', ');
	return total > Math.min(lines.length, WISE_LINES_SHOWN) ? `${shown}…` : shown;
}

/** Le nom du fichier refusé, avec l'archive d'où il vient s'il sort d'un zip. */
function fileOf(details: Details): string {
	const filename = typeof details?.filename === 'string' ? details.filename : '';
	const archive = typeof details?.archive === 'string' ? details.archive : null;
	return archive
		? i18nMsg('wise-import-error-file-in-archive', '« { $filename } » (archive « { $archive } »)', {
				filename,
				archive,
			})
		: i18nMsg('wise-import-error-file', '« { $filename } »', { filename });
}

/**
 * Les phrases d'un refus de l'import Wise. Un relevé refusé par le lecteur
 * (`WISE_CSV_INVALID`) se dit par sa cause, le fichier nommé ; un relevé exporté
 * sans les frais séparés (`WISE_CSV_FEES_NOT_SEPARATED`), par la consigne de
 * l'exporter de nouveau, option cochée ; deux fichiers d'une même devise, par
 * leurs noms. Tout autre refus : le message du serveur.
 */
export function wiseImportErrorLines(err: unknown): string[] {
	if (!isApiError(err)) return [errorMessageOf(err)];
	const details = err.details as Details;
	switch (err.code) {
		case 'WISE_CSV_FEES_NOT_SEPARATED': {
			const lines = numbersOf(details?.lines);
			const count = typeof details?.count === 'number' ? details.count : lines.length;
			const out = [
				i18nMsg(
					'wise-import-error-fees-not-separated',
					'Le relevé { $file } a été exporté sans les frais séparés : ses montants comprennent les frais de Wise, que Kesh ne peut alors ni passer en charge ni retrouver. Rien n’a été importé.',
					{ file: fileOf(details) },
				),
				i18nMsg(
					'wise-import-error-fees-not-separated-how',
					'Exportez-le de nouveau depuis Wise en cochant « Afficher les transactions dont les frais sont indiqués séparément », puis importez le nouveau fichier.',
				),
			];
			if (lines.length > 0) {
				out.push(
					i18nMsg('wise-import-error-fees-not-separated-lines', '{ $count } opération(s) à frais sans leur ligne de frais (lignes { $lines }).', {
						count: String(count),
						lines: wiseLinesList(lines, count),
					}),
				);
			}
			return out;
		}
		case 'WISE_DUPLICATE_CURRENCY_FILE': {
			const currency = typeof details?.currency === 'string' ? details.currency : '';
			return [
				i18nMsg(
					'wise-import-error-duplicate-currency-file',
					'Deux fichiers portent le relevé en { $currency } ({ $filenames }) : un seul fichier par devise. Rien n’a été importé.',
					{ currency, filenames: stringsOf(details?.filenames).join(', ') },
				),
			];
		}
		case 'WISE_CSV_INVALID':
			return wiseCsvInvalidLines(err, details);
		default:
			return [errorMessageOf(err)];
	}
}

/** Les phrases d'un relevé refusé par le lecteur (39-11a), selon `details.reason`. */
function wiseCsvInvalidLines(err: unknown, details: Details): string[] {
	const file = fileOf(details);
	const reason = typeof details?.reason === 'string' ? details.reason : '';
	switch (reason) {
		case 'WISE_CSV_MISSING_COLUMNS':
			return [
				i18nMsg(
					'wise-import-error-missing-columns',
					'Le fichier { $file } n’est pas un relevé Wise : colonnes manquantes ({ $columns }). Exportez de Wise le relevé du solde au format CSV.',
					{ file, columns: stringsOf(details?.columns).join(', ') },
				),
			];
		case 'WISE_CSV_DUPLICATE_COLUMN':
			return [
				i18nMsg('wise-import-error-duplicate-column', 'Le fichier { $file } porte une colonne en double : { $columns }.', {
					file,
					columns: stringsOf(details?.columns).join(', '),
				}),
			];
		case 'WISE_CSV_EMPTY':
			return [i18nMsg('wise-import-error-empty', 'Le fichier { $file } est vide : aucune ligne de données.', { file })];
		case 'WISE_CSV_ENCODING':
			return [i18nMsg('wise-import-error-encoding', 'L’encodage du fichier { $file } n’est pas reconnu.', { file })];
		case 'WISE_CSV_MIXED_CURRENCIES':
			return [
				i18nMsg(
					'wise-import-error-mixed-currencies',
					'Le fichier { $file } porte plusieurs devises ({ $currencies }) : Wise exporte un relevé par solde, un fichier par devise.',
					{ file, currencies: stringsOf(details?.currencies).join(', ') },
				),
			];
		case 'WISE_CSV_DUPLICATE_ID':
			return [
				i18nMsg(
					'wise-import-error-duplicate-id',
					'Le fichier { $file } porte deux fois l’identifiant { $id } dans le même sens (lignes { $lines }).',
					{
						file,
						id: typeof details?.id === 'string' ? details.id : '',
						lines: wiseLinesList(numbersOf(details?.lines)),
					},
				),
			];
		case 'WISE_CSV_RUNNING_BALANCE_BREAK':
			return [
				i18nMsg(
					'wise-import-error-running-balance',
					'Le fichier { $file } ne se suit pas : à la ligne { $line }, le solde courant vaut { $found } au lieu de { $expected }. Exportez de nouveau le relevé complet de la période.',
					{
						file,
						line: String(details?.line ?? ''),
						found: String(details?.found ?? ''),
						expected: String(details?.expected ?? ''),
					},
				),
			];
		case 'WISE_CSV_LINES':
			return [
				i18nMsg('wise-import-error-lines', 'Lignes en erreur dans le fichier { $file }, rien n’a été importé :', { file }),
				...lineErrorsOf(details).map((l) =>
					i18nMsg('wise-import-error-line', 'Ligne { $line }, { $field } : { $cause }', {
						line: String(l.line),
						field: l.field ?? '—',
						cause: wiseLineErrorLabel(l),
					}),
				),
			];
		default:
			return [errorMessageOf(err)];
	}
}

/**
 * L'avertissement de continuité d'un fichier (39-11b, AC-5) : le relevé ne
 * commence pas où finissait le précédent du compte. L'import est passé.
 */
export function wiseWarningLabel(w: WiseFileWarning, currency: string): string {
	if (w.code === 'gap') {
		return i18nMsg(
			'wise-import-warning-gap',
			'Le relevé ne commence pas où finissait le précédent : solde d’ouverture { $found } { $currency }, au lieu de { $expected } { $currency } à la clôture du relevé précédent. Une période manque peut-être : importez-la.',
			{ found: w.found, expected: w.expected, currency },
		);
	}
	return w.code;
}

/** La ligne du compte rendu d'un fichier : devise, lignes insérées, doublons, soldes. */
export function wiseFileReportLine(f: WiseFileReport): string {
	return i18nMsg(
		'wise-import-report-file',
		'{ $currency } : { $inserted } transaction(s) insérée(s), { $duplicates } doublon(s) ; solde de { $opening } à { $closing } { $currency }',
		{
			currency: f.currency,
			inserted: String(f.inserted),
			duplicates: String(f.duplicates),
			opening: f.openingBalance,
			closing: f.closingBalance,
		},
	);
}
