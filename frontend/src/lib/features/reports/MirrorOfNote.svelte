<script lang="ts">
	// Story 47-4 (AC-1) — sous le libellé de la ligne d'un compte miroir, au bilan et au
	// compte de résultat, la mention « Solde de 2270 AVS/AI/APG » (« Soldes de … » pour
	// plusieurs sources, séparées par des virgules). Rien pour une ligne sans `mirrorOf` :
	// elle est rendue exactement comme avant. Un seul composant pour les quatre tableaux à
	// plat et pour les lignes groupées (`GroupedReportRows`, revue d'intégration P1 de
	// l'Epic 47, H-1) (DRY).
	//
	// Depuis la reprise de l'Epic 55 (liens de compte des rapports) : le numéro de chaque
	// source est un lien vers **son** grand livre sur la période du rapport (`AccountLink`),
	// car c'est la source qui porte les écritures — le miroir, lui, n'en porte aucune. Le texte
	// de la mention ne change pas (revue d'intégration P1 de la PR de l'Epic 47, M-1).
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import AccountLink from './AccountLink.svelte';
	import type { AccountBalance } from './reports.types';

	interface Props {
		account: Pick<AccountBalance, 'accountNumber' | 'mirrorOf'>;
		/** Période du rapport (`AAAA-MM-JJ`), pour les liens des sources. */
		from: string;
		to: string;
	}
	let { account, from, to }: Props = $props();

	let sources = $derived(account.mirrorOf ?? []);
	// Le serveur pré-résout les messages Fluent sans arguments : un sélecteur sur `$count`
	// n'y survivrait pas. D'où deux clés, `-one` et `-other`, comme ailleurs dans Kesh.
	// La phrase se rend avec un repère à la place de `$sources`, puis se coupe autour de lui :
	// les sources s'intercalent en liens, dans l'ordre de la phrase de chaque langue.
	const REPERE = '\u0001';
	let morceaux = $derived.by(() => {
		const phrase =
			sources.length === 1
				? i18nMsg('reports-mirror-of-one', 'Solde de { $sources }', { sources: REPERE })
				: i18nMsg('reports-mirror-of-other', 'Soldes de { $sources }', { sources: REPERE });
		const i = phrase.indexOf(REPERE);
		return i < 0 ? { avant: phrase, apres: '' } : { avant: phrase.slice(0, i), apres: phrase.slice(i + 1) };
	});
</script>

{#if sources.length > 0}
	<span class="block text-xs text-gray-500" data-testid="mirror-of-{account.accountNumber}"
		>{morceaux.avant}{#each sources as s, i (s.accountId)}{#if i > 0}{', '}{/if}<AccountLink
				accountId={s.accountId}
				number={s.accountNumber}
				{from}
				{to}
			/> {s.accountName}{/each}{morceaux.apres}</span
	>
{/if}
