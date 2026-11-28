// Story 14-3c — tests Vitest pour BalanceSheetView.svelte.
// Couvre la présentation des fonds propres PAR RÔLE : section « Capitaux propres »
// dédiée groupée par rôle, distinction compte physique / ligne calculée (D1), et
// absence des comptes de fonds propres dans la table Passifs (partition D2).
//
// i18nMsg est mocké pour renvoyer le fallback → le sous-titre de rôle affiche la
// valeur brute du rôle (2e argument passé par le composant), suffisant pour asserter
// le regroupement sans dépendre des traductions.

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	// Story 47-4 — le repli, ses variables `{ $x }` remplacées (mention « Solde de … »).
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_m, k) => String(args[k] ?? '')) : fallback,
}));

import BalanceSheetView from './BalanceSheetView.svelte';
import type { BalanceSheetDto } from './reports.types';

/** Bilan avec : 1 actif, 1 dette réelle (Payable), 2 comptes de fonds propres
 *  physiques (Capital + Bénéfice reporté 2970 à 50'000), + 2 lignes calculées
 *  virtuelles (report 5'000, résultat 1'200) — collision D1 sur RetainedEarnings. */
function makeDto(overrides: Partial<BalanceSheetDto> = {}): BalanceSheetDto {
	return {
		period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
		assets: [
			{
				accountId: 1,
				accountNumber: '1000',
				accountName: 'Banque',
				accountType: 'Asset',
				active: true,
				balance: '56200',
				role: null,
			},
		],
		liabilities: [
			{
				accountId: 2,
				accountNumber: '2000',
				accountName: 'Fournisseurs',
				accountType: 'Liability',
				active: true,
				balance: '0',
				role: 'Payable',
			},
		],
		equity: [
			{
				accountId: 3,
				accountNumber: '2800',
				accountName: 'Capital social',
				accountType: 'Liability',
				active: true,
				balance: '20000',
				role: 'EquityCapital',
			},
			{
				accountId: 4,
				accountNumber: '2970',
				accountName: 'Bénéfice reporté (compte)',
				accountType: 'Liability',
				active: true,
				balance: '30000',
				role: 'RetainedEarnings',
			},
		],
		totalAssets: '56200',
		totalLiabilities: '0',
		totalEquity: '50000',
		retainedEarnings: '5000',
		equityResult: '1200',
		equationHolds: true,
		...overrides,
	};
}

describe('BalanceSheetView — fonds propres par rôle (Story 14-3c)', () => {
	it('affiche une section Capitaux propres dédiée avec les comptes physiques', () => {
		const { getAllByText, container } = render(BalanceSheetView, { dto: makeDto() });
		// Titre de section (fallback i18n).
		expect(getAllByText('Capitaux propres').length).toBeGreaterThan(0);
		// Les comptes physiques de fonds propres sont itemisés.
		expect(container.textContent).toContain('Capital social');
		expect(container.textContent).toContain('Bénéfice reporté (compte)');
	});

	it('distingue le compte physique RetainedEarnings de la ligne calculée (D1)', () => {
		const { container } = render(BalanceSheetView, { dto: makeDto() });
		// Ligne calculée explicitement marquée « (calculé) » (clé -calculated).
		expect(container.textContent).toContain('Résultat reporté (calculé)');
		// Le compte physique 2970 reste une ligne distincte sous son numéro.
		expect(container.textContent).toContain('2970');
	});

	it('exclut les comptes de fonds propres de la table Passifs (partition D2)', () => {
		const { container } = render(BalanceSheetView, { dto: makeDto() });
		const tables = container.querySelectorAll('table');
		// Ordre DOM : [0] Actifs, [1] Passifs, [2] Capitaux propres.
		expect(tables.length).toBe(3);
		const liabilities = tables[1].textContent ?? '';
		const equity = tables[2].textContent ?? '';
		// Le capital NE figure PAS dans les Passifs, mais dans Capitaux propres.
		expect(liabilities).not.toContain('Capital social');
		expect(liabilities).toContain('Fournisseurs');
		expect(equity).toContain('Capital social');
		expect(equity).toContain('2970');
	});

	it('affiche la section même sur un reclassement pur equity (report/résultat nuls)', () => {
		const dto = makeDto({
			assets: [],
			liabilities: [],
			totalAssets: '0',
			totalLiabilities: '0',
			retainedEarnings: '0',
			equityResult: '0',
			// equity conservé (2 comptes) → NON vide malgré actif/passif/virtuels nuls.
			totalEquity: '50000',
		});
		const { container, queryByRole } = render(BalanceSheetView, { dto });
		// Pas de message « rapport vide ».
		expect(queryByRole('status')).toBeNull();
		expect(container.textContent).toContain('Capital social');
	});
});

describe('BalanceSheetView — ligne calculée « Résultat reporté » (fix-bilan-resultat-reporte)', () => {
	// Revue P1, LOW-3 : la table se repère par son data-testid, non par sa position.
	const equityContent = (getByTestId: (id: string) => HTMLElement) =>
		getByTestId('balance-sheet-equity').textContent ?? '';

	// Revue P1, LOW-1 : un résidu sous le demi-centime s'arrondit à zéro, comme il
	// s'afficherait — la ligne est omise, ni « 0.00 » ni « -0.00 ».
	it.each(['0', '0.00', '-0', '0.004', '-0.004'])(
		'omet la ligne calculée quand le report vaut %s, total capitaux propres inchangé',
		(zero) => {
			// 50'000 (comptes) + report + 1'200 → 51'200.00 à l'affichage.
			const { getByTestId, queryByTestId } = render(BalanceSheetView, {
				dto: makeDto({ retainedEarnings: zero, totalAssets: '51200' }),
			});
			expect(queryByTestId('balance-sheet-retained-earnings')).toBeNull();
			const equity = equityContent(getByTestId);
			expect(equity).not.toContain('Résultat reporté');
			expect(equity).not.toContain('Perte reportée');
			expect(equity).not.toContain('-0.00');
			// Le compte physique de report et le résultat de l'exercice restent.
			expect(equity).toContain('2970');
			expect(equity).toContain("Bénéfice de l'exercice");
			expect(equity).toMatch(/51.200\.00/);
		},
	);

	it('affiche la ligne calculée non nulle (bénéfice reporté) avec son montant', () => {
		const { getByTestId } = render(BalanceSheetView, { dto: makeDto() });
		const row = getByTestId('balance-sheet-retained-earnings');
		expect(row.textContent).toContain('Résultat reporté (calculé)');
		expect(row.textContent).toMatch(/5.000\.00/);
	});

	it('affiche au demi-centime la ligne arrondie loin de zéro (« 0.01 »)', () => {
		const { getByTestId } = render(BalanceSheetView, {
			dto: makeDto({ retainedEarnings: '0.005' }),
		});
		const row = getByTestId('balance-sheet-retained-earnings');
		expect(row.textContent).toContain('Résultat reporté (calculé)');
		expect(row.textContent).toContain('0.01');
	});

	it('affiche la ligne calculée négative sous « Perte reportée »', () => {
		const { getByTestId } = render(BalanceSheetView, {
			dto: makeDto({ retainedEarnings: '-0.05' }),
		});
		const row = getByTestId('balance-sheet-retained-earnings');
		expect(row.textContent).toContain('Perte reportée');
		expect(row.textContent).toContain('-0.05');
	});
});

describe('BalanceSheetView — Story 37-4, compte en devise', () => {
	it('montre « USD 1’309.00 » à côté du nom d’un compte en devise, rien pour un compte CHF', () => {
		const base = makeDto();
		const usd = {
			...base.assets[0],
			accountId: 9,
			accountNumber: '1020',
			accountName: 'Banque USD',
			balance: '1195.00',
			currency: 'USD',
			balanceCurrency: '1309.0000',
		};
		const { getByTestId, queryByTestId } = render(BalanceSheetView, {
			dto: makeDto({ assets: [base.assets[0], usd] }),
		});
		expect(getByTestId('account-fx-1020').textContent).toBe('USD 1’309.00');
		expect(queryByTestId('account-fx-1000')).toBeNull();
	});

	it('mono-devise : aucune mention en devise', () => {
		const { container } = render(BalanceSheetView, { dto: makeDto() });
		expect(container.innerHTML).not.toContain('account-fx-');
	});
});

// Story 55-2 (AC-1, AC-16) — filet de non-régression du lien inchangé (passé par `AccountLink`) : `href`,
// titre et classe exacts (la classe devient `LINK_CLASS`, D17).
describe('BalanceSheetView — Story 55-2, liens de compte', () => {
	it('actif, passif, fonds propres : lien exact, titre, classe LINK_CLASS', async () => {
		const { LINK_CLASS } = await import('$lib/shared/navigation/link-class');
		const { getByRole } = render(BalanceSheetView, { dto: makeDto() });
		for (const [num, id] of [['1000', 1], ['2000', 2], ['2800', 3]] as const) {
			const a = getByRole('link', { name: num });
			expect(a.getAttribute('href')).toBe(`/reports?tab=general-ledger&accountId=${id}&from=2026-01-01&to=2026-12-31`);
			expect(a.getAttribute('title')).toBe('Voir le détail dans le grand livre');
			expect(a.getAttribute('class')).toBe(LINK_CLASS);
		}
	});
});

describe('BalanceSheetView — Story 47-4, mention « Solde de … » sous un compte miroir', () => {
	const miroirActif = {
		accountId: 40,
		accountNumber: '1040.1',
		accountName: 'Créances sociales',
		accountType: 'Asset' as const,
		active: true,
		balance: '1200.00',
		role: null,
		mirrorOf: [{ accountId: 22, accountNumber: '2270', accountName: 'AVS/AI/APG', balance: '-1200.00' }],
	};
	const miroirPassif = {
		accountId: 21,
		accountNumber: '2100.1',
		accountName: 'Banque à découvert',
		accountType: 'Liability' as const,
		active: true,
		balance: '1000.00',
		role: null,
		mirrorOf: [
			{ accountId: 12, accountNumber: '1020', accountName: 'Banque', balance: '-800.00' },
			{ accountId: 13, accountNumber: '1021', accountName: 'Banque 2', balance: '-200.00' },
		],
	};

	it('une source : « Solde de 2270 AVS/AI/APG » sous la ligne du miroir, rien sous les autres', () => {
		const dto = makeDto();
		const { getByTestId, queryByTestId } = render(BalanceSheetView, {
			dto: { ...dto, assets: [...dto.assets, miroirActif] },
		});
		const note = getByTestId('mirror-of-1040.1');
		expect(note.textContent).toBe('Solde de 2270 AVS/AI/APG');
		// Sous le libellé, dans la cellule du nom de la ligne du miroir.
		expect(note.closest('tr')?.getAttribute('data-testid')).toBe('report-row-1040.1');
		expect(note.closest('td')?.textContent).toContain('Créances sociales');
		// Texte seul, sans lien.
		expect(note.querySelector('a')).toBeNull();
		expect(note.closest('a')).toBeNull();
		// Ligne sans `mirrorOf` : aucune mention.
		expect(queryByTestId('mirror-of-1000')).toBeNull();
		expect(getByTestId('report-row-1000').textContent).not.toContain('Solde de');
	});

	it('plusieurs sources au passif : « Soldes de … », séparées par des virgules', () => {
		const dto = makeDto();
		const { getByTestId } = render(BalanceSheetView, {
			dto: { ...dto, liabilities: [...dto.liabilities, miroirPassif] },
		});
		expect(getByTestId('mirror-of-2100.1').textContent).toBe('Soldes de 1020 Banque, 1021 Banque 2');
		expect(getByTestId('report-row-2100.1')).toBeTruthy();
	});

	it('le numéro du miroir reste lié au grand livre ; les fonds propres ne portent ni mention ni testid de ligne', () => {
		const dto = makeDto();
		const { getByTestId, container } = render(BalanceSheetView, {
			dto: { ...dto, assets: [...dto.assets, miroirActif] },
		});
		const lien = getByTestId('report-row-1040.1').querySelector('a');
		expect(lien?.textContent).toBe('1040.1');
		expect(lien?.getAttribute('href')).toContain('/reports');
		const fondsPropres = container.querySelector('[data-testid="balance-sheet-equity"]');
		expect(fondsPropres?.querySelector('[data-testid^="mirror-of-"]')).toBeNull();
		expect(fondsPropres?.querySelector('[data-testid^="report-row-"]')).toBeNull();
	});

	it('sans aucun miroir : aucune mention dans tout le rapport', () => {
		const { container } = render(BalanceSheetView, { dto: makeDto() });
		expect(container.querySelector('[data-testid^="mirror-of-"]')).toBeNull();
		expect(container.textContent).not.toContain('Solde de');
	});
});
