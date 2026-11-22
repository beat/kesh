// Story 46-2 — la page des rapports et la vue groupée du bilan (AC-1, AC-3, AC-5,
// AC-6, AC-10) : vue restaurée avant la première requête, redemande à chaque
// changement sans effacer l'affichage, seule la dernière réponse retenue, écho
// mémorisé, export aux paramètres affichés, hiérarchie masquée sans paramètre.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import type { BalanceSheetDto, GroupingParams } from '$lib/features/reports/reports.types';

const { getBalanceSheetMock, getIncomeStatementMock, downloadReportMock } = vi.hoisted(() => ({
	getBalanceSheetMock: vi.fn(),
	getIncomeStatementMock: vi.fn(),
	downloadReportMock: vi.fn(async () => {}),
}));

vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/reports') } }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
}));
vi.mock('$lib/features/projects/projects.api', () => ({ listProjects: async () => [] }));
vi.mock('$lib/features/accounts/accounts.api', () => ({ fetchAccounts: async () => [] }));
vi.mock('$lib/features/reports/reports.api', async (importOriginal) => {
	const vrai = await importOriginal<typeof import('$lib/features/reports/reports.api')>();
	return {
		...vrai,
		downloadReport: downloadReportMock,
		getBalanceSheet: getBalanceSheetMock,
		getIncomeStatement: getIncomeStatementMock,
	};
});

import Page from './+page.svelte';

const data = {
	fiscalYears: [
		{
			id: 1,
			companyId: 7,
			name: '2026',
			startDate: '2026-01-01',
			endDate: '2026-12-31',
			status: 'Open',
			multiCurrency: false,
		},
	],
	companyName: 'Test SA',
	companyId: 7,
} as never;

const KEY = 'kesh-report-grouping:7:balance-sheet';

/** Bilan groupé : groupe 10 (id 90) ouvert ou fermé selon `open`, écho de `params`. */
function bilan(params: GroupingParams | undefined, open = true, echo?: Partial<GroupingParams>): BalanceSheetDto {
	const base: BalanceSheetDto = {
		period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
		assets: [
			{ accountId: 1, accountNumber: '1000', accountName: 'Caisse', accountType: 'Asset', active: true, balance: '100', role: null },
		],
		liabilities: [],
		equity: [],
		totalAssets: '100',
		totalLiabilities: '0',
		totalEquity: '0',
		retainedEarnings: '0',
		equityResult: '100',
		equationHolds: true,
	};
	if (!params) return base;
	return {
		...base,
		grouping: { ...params, maxDepth: 2, ...echo },
		groupedRows: {
			assets: [
				{ kind: 'group', depth: 1, accountId: 90, accountNumber: '10', accountName: 'Actifs circulants', balance: '100', open },
				...(open
					? [{ kind: 'account' as const, depth: 2, accountId: 1, accountNumber: '1000', accountName: 'Caisse', balance: '100', active: true }]
					: []),
			],
			liabilities: [],
			equity: [],
		},
	};
}

async function generer(r: ReturnType<typeof render>) {
	await fireEvent.click(r.getByRole('button', { name: 'Générer' }));
}

describe('page des rapports — vue groupée du bilan (Story 46-2)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorage.clear();
		getBalanceSheetMock.mockImplementation(async (_q, p?: GroupingParams) => bilan(p, true));
	});
	afterEach(() => vi.restoreAllMocks());

	it('première visite : bilan groupé, tous les comptes, sous-totaux sur le titre (AC-1)', async () => {
		const r = render(Page, { data });
		await generer(r);
		expect(getBalanceSheetMock).toHaveBeenCalledWith(expect.objectContaining({ fiscalYearId: 1 }), {
			detailLevel: 'all',
			subtotalPosition: 'header',
			open: [],
			closed: [],
		});
		await r.findByTestId('grouped-group-10');
		expect((r.getByTestId('grouping-shown') as HTMLInputElement).checked).toBe(true);
	});

	it('vue mémorisée relue AVANT la première requête (AC-10)', async () => {
		localStorage.setItem(
			KEY,
			JSON.stringify({ shown: true, detailLevel: 2, subtotalPosition: 'footer', open: [90, 555], closed: [] }),
		);
		const r = render(Page, { data });
		await generer(r);
		expect(getBalanceSheetMock.mock.calls[0][1]).toEqual({
			detailLevel: 2,
			subtotalPosition: 'footer',
			open: [90, 555],
			closed: [],
		});
	});

	it("groupe disparu : l'écho le retire de la mémoire après la réponse (AC-10)", async () => {
		localStorage.setItem(KEY, JSON.stringify({ shown: true, detailLevel: 1, subtotalPosition: 'header', open: [90, 555], closed: [] }));
		getBalanceSheetMock.mockImplementation(async (_q, p) => bilan(p, true, { open: [90] }));
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		expect(JSON.parse(localStorage.getItem(KEY)!).open).toEqual([90]);
	});

	it('mémoire illisible ou refusée : vue par défaut, sans erreur (AC-5)', async () => {
		vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
			throw new DOMException('refusé', 'SecurityError');
		});
		vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			throw new DOMException('refusé', 'SecurityError');
		});
		const r = render(Page, { data });
		await generer(r);
		expect(getBalanceSheetMock.mock.calls[0][1]).toMatchObject({ detailLevel: 'all' });
		await r.findByTestId('grouped-group-10');
		// la redemande et l'affichage marchent quand même
		await fireEvent.click(r.getByTestId('group-toggle-10'));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(2));
		expect(r.queryByRole('alert')).toBeNull();
	});

	it('clic sur un accordéon : mémorisé, redemandé avec closed, affichage gardé (AC-3)', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		let resoudre!: (v: BalanceSheetDto) => void;
		getBalanceSheetMock.mockImplementationOnce(() => new Promise((res) => (resoudre = res)));
		await fireEvent.click(r.getByTestId('group-toggle-10'));
		expect(getBalanceSheetMock.mock.calls[1][1]).toMatchObject({ open: [], closed: [90] });
		expect(JSON.parse(localStorage.getItem(KEY)!).closed).toEqual([90]);
		// pendant la redemande : l'affichage précédent reste, marqué occupé
		expect(r.getByTestId('grouped-account-1000')).toBeTruthy();
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('true');
		resoudre(bilan(getBalanceSheetMock.mock.calls[1][1], false, { closed: [90] }));
		await waitFor(() => expect(r.queryByTestId('grouped-account-1000')).toBeNull());
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false');
	});

	it('requêtes concurrentes : seule la dernière réponse s’affiche (AC-3)', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		const attentes: ((v: BalanceSheetDto) => void)[] = [];
		getBalanceSheetMock.mockImplementation(() => new Promise((res) => attentes.push(res)));
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		await fireEvent.change(r.getByTestId('grouping-position'), { target: { value: 'footer' } });
		expect(attentes).toHaveLength(2);
		// la seconde arrive d'abord, puis la première, périmée
		attentes[1](bilan({ detailLevel: 1, subtotalPosition: 'footer', open: [], closed: [] }, false));
		await waitFor(() => expect(r.queryByTestId('grouped-account-1000')).toBeNull());
		attentes[0](bilan({ detailLevel: 1, subtotalPosition: 'header', open: [], closed: [] }, true));
		await new Promise((res) => setTimeout(res, 0));
		expect(r.queryByTestId('grouped-account-1000')).toBeNull();
	});

	it('changement de niveau : listes vidées, mémorisé ; tout ouvrir et tout fermer', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(2));
		expect(getBalanceSheetMock.mock.calls[1][1]).toMatchObject({ detailLevel: 1, open: [], closed: [] });
		expect(JSON.parse(localStorage.getItem(KEY)!).detailLevel).toBe(1);
		await fireEvent.click(r.getByTestId('grouping-open-all'));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(3));
		expect(getBalanceSheetMock.mock.calls[2][1]).toMatchObject({ detailLevel: 'all' });
		await fireEvent.click(r.getByTestId('grouping-close-all'));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(4));
		expect(getBalanceSheetMock.mock.calls[3][1]).toMatchObject({ detailLevel: 1 });
	});

	it('position des sous-totaux mémorisée et transmise', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.change(r.getByTestId('grouping-position'), { target: { value: 'footer' } });
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(2));
		expect(getBalanceSheetMock.mock.calls[1][1]).toMatchObject({ subtotalPosition: 'footer' });
		expect(JSON.parse(localStorage.getItem(KEY)!).subtotalPosition).toBe('footer');
	});

	it('hiérarchie masquée : requête sans paramètre, écran à plat, choix gardés (C-46-2-2)', async () => {
		localStorage.setItem(KEY, JSON.stringify({ shown: true, detailLevel: 2, subtotalPosition: 'footer', open: [], closed: [] }));
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.click(r.getByTestId('grouping-shown'));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(2));
		expect(getBalanceSheetMock.mock.calls[1][1]).toBeUndefined();
		await waitFor(() => expect(r.queryByTestId('grouped-group-10')).toBeNull());
		expect(r.getByText('Caisse')).toBeTruthy();
		expect((r.getByTestId('grouping-level') as HTMLSelectElement).disabled).toBe(true);
		expect(JSON.parse(localStorage.getItem(KEY)!)).toMatchObject({ shown: false, detailLevel: 2, subtotalPosition: 'footer' });
	});

	it('export : les paramètres de la dernière réponse affichée (AC-6)', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.click(r.getByTestId('group-toggle-10'));
		await waitFor(() => expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false'));
		await fireEvent.click(r.getByRole('button', { name: 'Export CSV' }));
		await waitFor(() => expect(downloadReportMock).toHaveBeenCalledTimes(1));
		const args = downloadReportMock.mock.calls[0] as unknown[];
		expect(args[0]).toBe('balance-sheet');
		expect(args[2]).toBe('csv');
		expect(args[4]).toEqual({ detailLevel: 'all', subtotalPosition: 'header', open: [], closed: [90] });
	});

	it('sélecteur de niveau : 1 à maxDepth − 1, puis Tous les comptes', async () => {
		const r = render(Page, { data });
		expect((r.getByTestId('grouping-level') as HTMLSelectElement).disabled).toBe(true);
		await generer(r);
		await r.findByTestId('grouped-group-10');
		const select = r.getByTestId('grouping-level') as HTMLSelectElement;
		expect(select.disabled).toBe(false);
		expect([...select.options].map((o) => o.textContent)).toEqual(['Niveau 1', 'Tous les comptes']);
	});
});
