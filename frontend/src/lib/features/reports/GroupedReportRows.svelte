<script lang="ts" module>
	import type { GroupedRow } from './reports.types';

	/**
	 * Une colonne de montant d'une ligne groupée (T3) : le bilan et le compte de
	 * résultat en ont une (le solde), la balance (46-6b) en aura quatre. Le solde en
	 * devise reste dans la cellule du libellé, comme `AccountCurrencyBalance`.
	 */
	export interface GroupedAmountColumn {
		/** Identifiant stable de la colonne (clé de rendu). */
		id: string;
		/** Le montant exact de la ligne pour cette colonne (chaîne décimale de l'API). */
		amount: (row: GroupedRow) => string;
	}

	/**
	 * La colonne unique du bilan et du compte de résultat : le solde de la ligne
	 * (revue P1 L3 — une seule définition pour les deux vues).
	 */
	export const BALANCE_COLUMNS: GroupedAmountColumn[] = [
		{ id: 'balance', amount: (row) => row.balance },
	];

	/** Retrait d'une profondeur, en rem ; borné pour que le nom garde sa place. */
	const INDENT_REM = 1.25;
	const INDENT_MAX_DEPTH = 10;

	/** Retrait de la première cellule d'une ligne de profondeur `depth` (1 = racine). */
	export function indentOf(depth: number): string {
		const steps = Math.min(Math.max(depth - 1, 0), INDENT_MAX_DEPTH);
		return `${0.5 + steps * INDENT_REM}rem`;
	}

	/**
	 * Règle de rendu de la 46-1, unique pour l'écran, le PDF et le CSV : une ligne
	 * `group` porte son montant si les sous-totaux sont sur le titre **ou** si le
	 * groupe est fermé ; sinon le montant va à sa ligne `subtotal`. Les lignes
	 * `account`, `own` et `subtotal` portent toujours le leur.
	 */
	export function showsAmount(row: GroupedRow, subtotalPosition: 'header' | 'footer'): boolean {
		if (row.kind !== 'group') return true;
		return subtotalPosition === 'header' || row.open === false;
	}
</script>

<script lang="ts">
	// Story 46-2 (T3, AC-4) — les lignes groupées d'une section d'un rapport, dans
	// l'ordre de `groupedRows` (46-1) : retrait par profondeur, titre de groupe en
	// gras avec son accordéon, solde propre, sous-total en pied. Partagé par le
	// bilan et le compte de résultat, et repris par la balance (46-6b) : il ne
	// connaît que des lignes et des colonnes, jamais un rapport. Les montants exacts
	// de l'API (sous-totaux compris) s'arrondissent au centime à l'affichage
	// seulement (`formatCurrencyFigure` → `roundToCents`, sans « -0.00 »).
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import AccountCurrencyBalance from './AccountCurrencyBalance.svelte';
	import { formatCurrencyFigure, ledgerHref } from './reports.api';
	import type { SubtotalPosition } from './reports.types';

	interface Props {
		rows: GroupedRow[];
		subtotalPosition: SubtotalPosition;
		columns: GroupedAmountColumn[];
		/** Période affichée : le numéro d'un compte mène à son grand livre. */
		period: { startDate: string; endDate: string };
		/** Clic sur l'accordéon d'un groupe ; l'écran redemande le rapport. */
		onToggle: (row: GroupedRow) => void;
	}
	let { rows, subtotalPosition, columns, period, onToggle }: Props = $props();

	/** Libellé de l'accordéon : ce que fait le clic, groupe nommé. */
	function toggleLabel(row: GroupedRow): string {
		return row.open
			? i18nMsg('reports-groups-toggle-close', 'Fermer le groupe { $number } { $name }', {
					number: row.accountNumber,
					name: row.accountName,
				})
			: i18nMsg('reports-groups-toggle-open', 'Ouvrir le groupe { $number } { $name }', {
					number: row.accountNumber,
					name: row.accountName,
				});
	}
</script>

<!-- Clé : genre + identifiant — la ligne `subtotal` reprend l'identifiant de son groupe. -->
{#each rows as row (`${row.kind}-${row.accountId}`)}
	{@const shown = showsAmount(row, subtotalPosition)}
	<tr
		class:font-semibold={row.kind === 'group' || row.kind === 'subtotal'}
		class:border-t={row.kind === 'subtotal'}
		class:opacity-60={row.active === false}
		data-testid="grouped-{row.kind}-{row.accountNumber}"
		data-depth={row.depth}
	>
		<td class="py-1 pr-2 font-mono whitespace-nowrap" style:padding-left={indentOf(row.depth)}>
			{#if row.kind === 'group'}
				<button
					type="button"
					class="mr-1 inline-flex h-5 w-5 items-center justify-center rounded text-gray-600 hover:bg-gray-200 focus:ring-2 focus:ring-indigo-500 focus:outline-none"
					aria-expanded={row.open === true}
					aria-label={toggleLabel(row)}
					data-testid="group-toggle-{row.accountNumber}"
					onclick={() => onToggle(row)}
					><span aria-hidden="true">{row.open ? '▾' : '▸'}</span></button
				>{row.accountNumber}
			{:else if row.kind === 'subtotal'}
				<span class="inline-block w-6" aria-hidden="true"></span>
			{:else}
				<span class="mr-1 inline-block w-5" aria-hidden="true"></span><a
					class="text-indigo-700 hover:underline"
					href={ledgerHref(row.accountId, period.startDate, period.endDate)}
					title={i18nMsg('reports-ledger-open-from-balance', 'Voir le détail dans le grand livre')}
					>{row.accountNumber}</a
				>
			{/if}
		</td>
		<!-- Le libellé suit le même retrait que le numéro : un pied de groupe se lit au rang de son groupe. -->
		<td class="py-1 pr-2" style:padding-left={indentOf(row.depth)}>
			{#if row.kind === 'subtotal'}
				{i18nMsg('reports-groups-subtotal', 'Total { $name }', { name: row.accountName })}
			{:else if row.kind === 'own'}
				{i18nMsg('reports-groups-own-balance', '{ $name } (solde propre)', {
					name: row.accountName,
				})}
			{:else}
				{row.accountName}
			{/if}
			{#if shown}
				<AccountCurrencyBalance
					account={row}
					testIdPrefix={row.kind === 'group' || row.kind === 'subtotal' ? 'group-fx' : 'account-fx'}
				/>
			{/if}
		</td>
		{#each columns as col (col.id)}
			<td class="px-2 py-1 text-right font-mono" data-testid="grouped-amount-{col.id}"
				>{shown ? formatCurrencyFigure(col.amount(row)) : ''}</td
			>
		{/each}
	</tr>
{/each}
