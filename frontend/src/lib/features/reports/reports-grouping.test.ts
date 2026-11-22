// Story 46-2 — état, transitions et mémoire de la vue groupée (AC-2, AC-3, AC-5, AC-10).

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
	closeAll,
	defaultGroupingView,
	detailLevelOptions,
	groupingParamsOf,
	groupingStorageKey,
	openAll,
	readGroupingView,
	sanitizeGroupingView,
	toggleGroup,
	withDetailLevel,
	withEcho,
	writeGroupingView,
	type GroupingView,
} from './reports-grouping';

const vue = (partiel: Partial<GroupingView> = {}): GroupingView => ({
	...defaultGroupingView(),
	...partiel,
});

describe('vue par défaut (AC-1)', () => {
	it('hiérarchie montrée, tous les comptes, sous-totaux sur le titre, aucune liste', () => {
		expect(defaultGroupingView()).toEqual({
			shown: true,
			detailLevel: 'all',
			subtotalPosition: 'header',
			open: [],
			closed: [],
		});
	});
});

describe('paramètres de requête', () => {
	it('hiérarchie masquée : aucun paramètre (écran à plat, C-46-2-2)', () => {
		expect(groupingParamsOf(vue({ shown: false, detailLevel: 2, open: [4] }))).toBeUndefined();
	});
	it('hiérarchie montrée : niveau, position et listes', () => {
		expect(
			groupingParamsOf(vue({ detailLevel: 2, subtotalPosition: 'footer', open: [4], closed: [9] })),
		).toEqual({ detailLevel: 2, subtotalPosition: 'footer', open: [4], closed: [9] });
	});
});

describe('règle des clics (AC-3)', () => {
	it('ouvrir un groupe fermé : ajouté à open, retiré de closed', () => {
		expect(toggleGroup(vue({ closed: [7, 8] }), 7, false)).toMatchObject({ open: [7], closed: [8] });
	});
	it('fermer un groupe ouvert : ajouté à closed, retiré de open', () => {
		expect(toggleGroup(vue({ open: [7, 3] }), 7, true)).toMatchObject({ open: [3], closed: [7] });
	});
	it('changer de niveau vide les deux listes', () => {
		expect(withDetailLevel(vue({ open: [1], closed: [2] }), 3)).toMatchObject({
			detailLevel: 3,
			open: [],
			closed: [],
		});
	});
	it('tout ouvrir = tous les comptes, tout fermer = niveau 1, listes vidées (C-46-2-3)', () => {
		expect(openAll(vue({ detailLevel: 1, open: [1] }))).toMatchObject({ detailLevel: 'all', open: [] });
		expect(closeAll(vue({ closed: [2] }))).toMatchObject({ detailLevel: 1, closed: [] });
	});
	it("l'écho de l'API remplace les listes (groupe disparu retiré, AC-10)", () => {
		const v = withEcho(vue({ detailLevel: 2, open: [5, 999] }), {
			detailLevel: 2,
			subtotalPosition: 'header',
			maxDepth: 3,
			open: [5],
			closed: [],
		});
		expect(v.open).toEqual([5]);
		expect(v.detailLevel).toBe(2);
	});
});

describe('niveaux proposés (AC-2)', () => {
	it('1 à maxDepth − 1 ; « Tous les comptes » est ajouté par le sélecteur', () => {
		expect(detailLevelOptions(4, 'all')).toEqual([1, 2, 3]);
	});
	it('plan sans groupe ou rapport vide : aucun niveau numéroté', () => {
		expect(detailLevelOptions(1, 'all')).toEqual([]);
		expect(detailLevelOptions(0, 'all')).toEqual([]);
	});
	it('borné à 20', () => {
		expect(detailLevelOptions(40, 'all')).toHaveLength(20);
	});
	it('un niveau mémorisé plus profond que le plan reste proposé', () => {
		expect(detailLevelOptions(3, 5)).toEqual([1, 2, 5]);
	});
});

describe('relecture de la mémoire (AC-5)', () => {
	it('champ invalide ignoré, les autres gardés', () => {
		expect(
			sanitizeGroupingView({ shown: false, detailLevel: 21, subtotalPosition: 'footer', open: [3] }),
		).toEqual({ shown: false, detailLevel: 'all', subtotalPosition: 'footer', open: [3], closed: [] });
	});
	it('niveau hors bornes ou non entier ignoré', () => {
		for (const n of [0, -1, 2.5, '2', 'ALL', null]) {
			expect(sanitizeGroupingView({ detailLevel: n }).detailLevel).toBe('all');
		}
		expect(sanitizeGroupingView({ detailLevel: 20 }).detailLevel).toBe(20);
	});
	it('liste non numérique, ou de plus de 500 identifiants, ignorée', () => {
		expect(sanitizeGroupingView({ open: ['a'] }).open).toEqual([]);
		expect(sanitizeGroupingView({ open: [1.5] }).open).toEqual([]);
		const longue = Array.from({ length: 501 }, (_, i) => i + 1);
		expect(sanitizeGroupingView({ closed: longue }).closed).toEqual([]);
		expect(sanitizeGroupingView({ closed: longue.slice(0, 500) }).closed).toHaveLength(500);
	});
	it('un identifiant dans les deux listes : les deux ignorées (400 sinon)', () => {
		const v = sanitizeGroupingView({ open: [1, 2], closed: [2] });
		expect(v.open).toEqual([]);
		expect(v.closed).toEqual([]);
	});
	it('ni objet ni valeur : vue par défaut', () => {
		expect(sanitizeGroupingView(null)).toEqual(defaultGroupingView());
		expect(sanitizeGroupingView('x')).toEqual(defaultGroupingView());
	});
});

describe('mémoire du navigateur (AC-10)', () => {
	beforeEach(() => localStorage.clear());
	afterEach(() => vi.restoreAllMocks());

	it('clé distincte par société et par rapport', () => {
		expect(groupingStorageKey(3, 'balance-sheet')).toBe('kesh-report-grouping:3:balance-sheet');
		writeGroupingView(3, 'balance-sheet', vue({ detailLevel: 2 }));
		writeGroupingView(3, 'income-statement', vue({ detailLevel: 1 }));
		writeGroupingView(4, 'balance-sheet', vue({ shown: false }));
		expect(readGroupingView(3, 'balance-sheet').detailLevel).toBe(2);
		expect(readGroupingView(3, 'income-statement').detailLevel).toBe(1);
		expect(readGroupingView(4, 'balance-sheet').shown).toBe(false);
		expect(readGroupingView(5, 'balance-sheet')).toEqual(defaultGroupingView());
	});
	it('aller-retour complet : position, hiérarchie et listes', () => {
		const v = vue({ shown: false, detailLevel: 3, subtotalPosition: 'footer', open: [8], closed: [9] });
		writeGroupingView(1, 'balance-sheet', v);
		expect(readGroupingView(1, 'balance-sheet')).toEqual(v);
	});
	it('stockage vide ou illisible : vue par défaut', () => {
		expect(readGroupingView(1, 'balance-sheet')).toEqual(defaultGroupingView());
		localStorage.setItem(groupingStorageKey(1, 'balance-sheet'), '{pas du json');
		expect(readGroupingView(1, 'balance-sheet')).toEqual(defaultGroupingView());
	});
	it('stockage refusé : vue par défaut en lecture, écriture sans erreur', () => {
		vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
			throw new DOMException('refusé', 'SecurityError');
		});
		vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			throw new DOMException('refusé', 'SecurityError');
		});
		expect(readGroupingView(1, 'balance-sheet')).toEqual(defaultGroupingView());
		expect(() => writeGroupingView(1, 'balance-sheet', vue())).not.toThrow();
	});
	it('société inconnue : ni lecture ni écriture', () => {
		const set = vi.spyOn(Storage.prototype, 'setItem');
		writeGroupingView(null, 'balance-sheet', vue({ detailLevel: 2 }));
		expect(set).not.toHaveBeenCalled();
		expect(readGroupingView(null, 'balance-sheet')).toEqual(defaultGroupingView());
	});
});
