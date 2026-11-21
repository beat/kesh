<script lang="ts">
	// Story 47-4 (AC-1) — sous le libellé de la ligne d'un compte miroir, au bilan et au
	// compte de résultat, la mention « Solde de 2270 AVS/AI/APG » (« Soldes de … » pour
	// plusieurs sources, séparées par des virgules). Texte seul, sans lien : les sources
	// n'ont pas de ligne au rapport. Rien pour une ligne sans `mirrorOf` : elle est rendue
	// exactement comme avant. Un seul composant pour les quatre tableaux (DRY).
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { AccountBalance } from './reports.types';

	interface Props {
		account: AccountBalance;
	}
	let { account }: Props = $props();

	let sources = $derived(account.mirrorOf ?? []);
	// Le serveur pré-résout les messages Fluent sans arguments : un sélecteur sur `$count`
	// n'y survivrait pas. D'où deux clés, `-one` et `-other`, comme ailleurs dans Kesh.
	let mention = $derived.by(() => {
		const jointes = sources.map((s) => `${s.accountNumber} ${s.accountName}`).join(', ');
		return sources.length === 1
			? i18nMsg('reports-mirror-of-one', 'Solde de { $sources }', { sources: jointes })
			: i18nMsg('reports-mirror-of-other', 'Soldes de { $sources }', { sources: jointes });
	});
</script>

{#if sources.length > 0}
	<span
		class="block text-xs text-gray-500"
		data-testid="mirror-of-{account.accountNumber}">{mention}</span
	>
{/if}
