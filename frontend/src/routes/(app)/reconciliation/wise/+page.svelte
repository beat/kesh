<!--
  Story 39-11d (AC-2) — la page Wise, sur la page commune des prestataires
  (`ProviderReviewPage`, 40-7d) : choix du compte Wise, import d'un geste de
  ses relevés CSV — un par solde, donc par devise — ou du zip que Wise fait
  télécharger, tel quel (39-11e), et le compte rendu **par fichier** (archive,
  lignes insérées, doublons, soldes, avertissement de continuité, entrées du
  zip non lues).

  La consigne d'export est à l'écran : un relevé exporté sans l'option « frais
  indiqués séparément » est refusé (`WISE_CSV_FEES_NOT_SEPARATED`), avec la
  même consigne.

  ⚠️ Pas encore de revue ni d'acceptation : elles viennent de la revue Wise
  (Story 39-11c, `GET /api/v1/wise/review`, `POST /api/v1/wise/accept`), qui
  n'est pas dans cette branche. Sans `getReview`, la page commune n'affiche ni
  exercice ni propositions.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { wiseAccountLabels } from '$lib/features/bank-accounts/provider';
	import ProviderReviewPage from '$lib/features/provider/ProviderReviewPage.svelte';
	import { importWiseFiles } from '$lib/features/wise/wise.api';
	import { wiseFileReportLine, wiseImportErrorLines, wiseWarningLabel } from '$lib/features/wise/wise-labels';
	import type { WiseImportResponse } from '$lib/features/wise/wise.types';
</script>

{#snippet importReportMain(report: WiseImportResponse)}
	<ul class="mt-2 space-y-2" data-testid="wise-import-report-files">
		{#each report.files as f, i (i)}
			<li data-testid="wise-import-report-file-{f.currency}">
				<p class="font-medium" data-testid="wise-import-report-file-name">
					{f.archive
						? i18nMsg('wise-import-report-file-in-archive', '{ $filename } (archive { $archive })', {
								filename: f.filename,
								archive: f.archive,
							})
						: f.filename}
				</p>
				<p data-testid="wise-import-report-file-line">{wiseFileReportLine(f)}</p>
				{#if f.importId === null}
					<p class="text-text-muted" data-testid="wise-import-report-file-all-known">
						{i18nMsg('wise-import-report-all-known', 'Toutes les lignes étaient déjà importées : aucun import créé.')}
					</p>
				{/if}
				{#each f.warnings as w, j (j)}
					<p class="text-amber-800" role="status" data-testid="wise-import-report-warning-{w.code}">
						{wiseWarningLabel(w, f.currency)}
					</p>
				{/each}
			</li>
		{/each}
	</ul>
	{#if report.ignoredEntries.length > 0}
		<p class="mt-2">{i18nMsg('wise-import-report-ignored', 'Entrées du zip non lues (ni relevé CSV à la racine) :')}</p>
		<ul class="mt-1 list-disc pl-5" data-testid="wise-import-report-ignored">
			{#each report.ignoredEntries as e, i (i)}
				<li>{e.entry}</li>
			{/each}
		</ul>
	{/if}
{/snippet}

<ProviderReviewPage
	testId="wise"
	fileAccept=".csv,text/csv,.zip,application/zip"
	multiple
	texts={{
		title: i18nMsg('wise-page-title', 'Wise'),
		subtitle: i18nMsg(
			'wise-page-subtitle',
			'Importez les relevés d’un compte Wise : le zip que Wise vous fait télécharger, tel quel, ou ses fichiers CSV, un par devise. Chaque relevé va au compte bancaire de sa devise. Rien n’est écrit dans les livres à l’import.',
		),
		noAccount: i18nMsg(
			'wise-no-account',
			'Aucun compte Wise. Créez un compte bancaire par devise (un par solde Wise), case « Compte Wise » cochée, dans les comptes bancaires.',
		),
		accountLabel: i18nMsg('wise-label-account', 'Compte Wise'),
		importButton: i18nMsg('wise-import-button', 'Importer des relevés Wise (zip ou CSV)'),
		importHelp: i18nMsg(
			'wise-import-help',
			'Dans Wise, ouvrez l’export des relevés : choisissez la période et le format CSV, et cochez « Afficher les transactions dont les frais sont indiqués séparément » — sans cette option, le relevé est refusé. Donnez ensuite ici le zip téléchargé, tel quel.',
		),
		idColumn: i18nMsg('wise-col-id', 'Identifiant Wise'),
	}}
	api={{ importFiles: importWiseFiles }}
	accountLabels={wiseAccountLabels}
	importErrorLines={wiseImportErrorLines}
	{importReportMain}
/>
