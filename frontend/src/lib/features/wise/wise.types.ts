/**
 * Story 39-11d (AC-2) — DTO de l'écran Wise, miroir de la réponse `201` de
 * `POST /api/v1/wise-imports` (39-11b ; zip de Wise, 39-11e). À la différence
 * de PayPal et de Stripe, le compte rendu se tient **par fichier** (Wise exporte
 * un relevé par solde, donc par devise), avec les soldes du relevé, son
 * avertissement de continuité et, pour un zip, l'archive d'où il vient.
 * Montants en chaînes décimales.
 */
import type { ProviderImportResponseBase } from '$lib/features/provider/provider-review.types';

/**
 * Avertissement d'un fichier (39-11b, AC-5) : `gap`, le relevé ne commence pas
 * où finissait le précédent du compte (`expected` = sa clôture, `found` =
 * l'ouverture du fichier). L'import passe.
 */
export interface WiseFileWarning {
	code: 'gap' | string;
	expected: string;
	found: string;
	previousImportId: number;
}

/** Le compte rendu d'un fichier importé. */
export interface WiseFileReport {
	/** Nom du fichier (pour un zip : le nom de l'entrée CSV). */
	filename: string;
	/** Le zip d'où vient le fichier, `null` pour un fichier envoyé à part (39-11e). */
	archive: string | null;
	currency: string;
	bankAccountId: number;
	/** `null` quand toutes les lignes du fichier étaient déjà connues. */
	importId: number | null;
	/** Lignes lues (`inserted + duplicates`). */
	rows: number;
	inserted: number;
	duplicates: number;
	openingBalance: string;
	closingBalance: string;
	warnings: WiseFileWarning[];
}

/** Une entrée d'un zip que l'import n'a pas lue (dossier, autre extension : 39-11e, D3). */
export interface WiseIgnoredEntry {
	archive: string;
	entry: string;
}

/** Réponse `201` de `POST /api/v1/wise-imports`. */
export interface WiseImportResponse extends ProviderImportResponseBase {
	files: WiseFileReport[];
	ignoredEntries: WiseIgnoredEntry[];
}
