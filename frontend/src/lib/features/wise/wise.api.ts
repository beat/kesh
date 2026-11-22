/**
 * Story 39-11d (AC-2) — client de la route d'import Wise (39-11b, zip 39-11e),
 * sur le client commun aux prestataires.
 */
import { importProviderFiles } from '$lib/features/provider/provider-review.api';
import type { WiseImportResponse } from './wise.types';

/**
 * `POST /api/v1/wise-imports` (multipart : un ou plusieurs `file`, `account`) —
 * importe d'un geste les relevés CSV du compte Wise `account`, un par devise,
 * ou le zip que Wise fait télécharger, tel quel. Tout ou rien : un fichier
 * refusé refuse l'envoi entier. Refus (`WISE_CSV_INVALID`, `WISE_ZIP_INVALID`,
 * `WISE_CSV_FEES_NOT_SEPARATED`, …) : `ApiError`, à traduire par
 * `wiseImportErrorLines` (`wise-labels.ts`).
 */
export async function importWiseFiles(files: readonly File[], account: string): Promise<WiseImportResponse> {
	return importProviderFiles<WiseImportResponse>('wise', files, account);
}
