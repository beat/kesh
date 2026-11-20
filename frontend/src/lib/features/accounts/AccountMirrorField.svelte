<script lang="ts">
	/**
	 * Story 47-2 (AC-2, AC-3) — le champ « Compte miroir » du dialogue d'un
	 * compte, en création et en modification : un `<select>` **natif** (comme
	 * `AccountCurrencyFields`), l'option *Aucun* et les candidats de
	 * `mirrorCandidates`, une aide et, sous le champ, le refus
	 * `MIRROR_ACCOUNT_INVALID` du serveur tel quel.
	 *
	 * Quand le compte modifié est lui-même le miroir d'autres comptes du plan
	 * chargé, le `<select>` cède la place à une phrase qui les nomme : le parent
	 * n'envoie alors pas le champ (sa valeur ne change pas).
	 *
	 * ⚠️ Jamais `AccountAutocomplete`, qui écarte les comptes en devise : un
	 * compte miroir admet un compte de toute devise (47-1, R8).
	 */
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { AccountResponse, AccountType } from './accounts.types';
	import { mirrorCandidates, mirrorSourceNumbers } from './mirror';

	interface Props {
		/** La valeur du `<select>` : `''` pour « Aucun », l'identifiant en texte sinon. */
		value: string;
		// fx-filtre-comptes : exempté — un compte miroir admet un compte de toute devise
		accounts: AccountResponse[];
		/** Le compte modifié, ou `null` à la création. */
		account: AccountResponse | null;
		/** Le type choisi dans le formulaire. */
		accountType: AccountType;
		/** Préfixe des `id` et `data-testid` (`account-create` / `account-edit`). */
		idPrefix: string;
		/** Refus `MIRROR_ACCOUNT_INVALID` du serveur (`err.message`). */
		error?: string;
	}

	let { value = $bindable(), accounts, account, accountType, idPrefix, error = '' }: Props = $props();

	let sources = $derived(account ? mirrorSourceNumbers(account.id, accounts) : []);
	let candidates = $derived(mirrorCandidates(account, { accountType }, accounts));

	const SELECT_CLASS = 'mt-1 w-full rounded-md border border-border bg-white px-3 py-2 text-sm';
</script>

{#if sources.length > 0}
	<div>
		<p class="text-sm font-medium text-text">{i18nMsg('accounts-mirror-label', 'Compte miroir')}</p>
		<p class="mt-1 text-sm text-text-muted" data-testid="{idPrefix}-dialog-mirror-of">
			{i18nMsg('accounts-mirror-of', 'Ce compte est le compte miroir de { $numbers }.', {
				numbers: sources.join(', '),
			})}
		</p>
	</div>
{:else}
	<div>
		<label for="{idPrefix}-mirror" class="text-sm font-medium text-text">
			{i18nMsg('accounts-mirror-label', 'Compte miroir')}
		</label>
		<select
			id="{idPrefix}-mirror"
			class={SELECT_CLASS}
			bind:value
			aria-describedby="{idPrefix}-mirror-help"
			aria-invalid={error ? 'true' : undefined}
			data-testid="{idPrefix}-dialog-mirror"
		>
			<option value="">{i18nMsg('accounts-mirror-none', 'Aucun')}</option>
			{#each candidates as a (a.id)}
				<option value={String(a.id)}>{a.number} {a.name}</option>
			{/each}
		</select>
		<p id="{idPrefix}-mirror-help" class="mt-1 text-xs text-text-muted">
			{i18nMsg(
				'accounts-mirror-help',
				'Le compte miroir est un compte de présentation : non postable, sans rôle ni écriture. Le solde de ce compte, s’il est de sens inverse, y paraît dans le bilan ou le compte de résultat.',
			)}
		</p>
		{#if error}
			<p class="mt-1 text-xs text-destructive" role="alert" data-testid="{idPrefix}-dialog-mirror-error">
				{error}
			</p>
		{/if}
	</div>
{/if}
