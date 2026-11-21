<script lang="ts">
	/**
	 * Banc de test de `AccountMirrorField` (Story 47-2, L3) : lie `value` comme
	 * le fait la page et expose un bouton qui change le type du formulaire.
	 * Non utilisé en production.
	 */
	import AccountMirrorField from './AccountMirrorField.svelte';
	import type { AccountResponse, AccountType } from './accounts.types';

	let { accounts }: { accounts: AccountResponse[] } = $props();
	let value = $state('');
	let accountType = $state<AccountType>('Asset');
</script>

<AccountMirrorField bind:value {accounts} account={null} {accountType} idPrefix="h" />
<output data-testid="h-state">{value}</output>
<button type="button" data-testid="h-liability" onclick={() => (accountType = 'Liability')}>type</button>
