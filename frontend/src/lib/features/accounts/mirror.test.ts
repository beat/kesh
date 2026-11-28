// Story 47-2 (AC-2, AC-6) — les candidats du compte miroir et les champs envoyés.

import { describe, it, expect } from 'vitest';
import type { AccountResponse, AccountType } from './accounts.types';
import {
	mirrorCandidates,
	mirrorFieldErrorOf,
	mirrorFieldForCreate,
	mirrorFieldForUpdate,
	mirrorSourceNumbers,
	oppositeAccountType,
} from './mirror';

function acc(partial: Partial<AccountResponse> & { id: number; number: string }): AccountResponse {
	return {
		companyId: 1,
		name: `Compte ${partial.number}`,
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: false,
		letterable: false,
		version: 1,
		createdAt: '',
		updatedAt: '',
		...partial,
	};
}

// La source et des cibles de toutes les sortes, dans l'ordre du plan.
const SOURCE = acc({ id: 1, number: '2270', accountType: 'Liability', postable: true });
const ACTIF = acc({ id: 2, number: '1040.1', accountType: 'Asset' });
const ACTIF_IMPUTABLE = acc({ id: 3, number: '1041', accountType: 'Asset', postable: true });
const ACTIF_ROLE = acc({ id: 4, number: '1100', accountType: 'Asset', role: 'Receivable' });
const ACTIF_ARCHIVE = acc({ id: 5, number: '1042', accountType: 'Asset', active: false });
const ACTIF_AVEC_MIROIR = acc({ id: 6, number: '1043', accountType: 'Asset', mirrorAccountId: 9 });
const PASSIF = acc({ id: 7, number: '2100.1', accountType: 'Liability' });
const CHARGE = acc({ id: 8, number: '6900.1', accountType: 'Expense' });
const PRODUIT = acc({ id: 9, number: '6950.2', accountType: 'Revenue' });
const PLAN = [SOURCE, ACTIF, ACTIF_IMPUTABLE, ACTIF_ROLE, ACTIF_ARCHIVE, ACTIF_AVEC_MIROIR, PASSIF, CHARGE, PRODUIT];

const ids = (list: AccountResponse[]) => list.map((a) => a.id);

describe('oppositeAccountType', () => {
	it.each<[AccountType, AccountType]>([
		['Asset', 'Liability'],
		['Liability', 'Asset'],
		['Revenue', 'Expense'],
		['Expense', 'Revenue'],
	])('%s ↔ %s', (t, o) => {
		expect(oppositeAccountType(t)).toBe(o);
	});
});

describe('mirrorCandidates', () => {
	it.each<[AccountType, number[]]>([
		// Passif → actifs non postables, sans rôle, actifs, sans miroir.
		['Liability', [ACTIF.id]],
		['Asset', [PASSIF.id]],
		['Revenue', [CHARGE.id]],
		['Expense', [PRODUIT.id]],
	])('type %s : seulement l’autre côté, compte de présentation', (type, attendus) => {
		const source = acc({ id: 100, number: '9999', accountType: type, postable: true });
		expect(ids(mirrorCandidates(source, { accountType: type }, [source, ...PLAN]))).toEqual(attendus);
	});

	it('écarte le compte postable, à rôle, archivé, qui a déjà un miroir', () => {
		const rendu = ids(mirrorCandidates(SOURCE, { accountType: 'Liability' }, PLAN));
		expect(rendu).not.toContain(ACTIF_IMPUTABLE.id);
		expect(rendu).not.toContain(ACTIF_ROLE.id);
		expect(rendu).not.toContain(ACTIF_ARCHIVE.id);
		expect(rendu).not.toContain(ACTIF_AVEC_MIROIR.id);
	});

	it('ne propose jamais le compte lui-même', () => {
		const lui = acc({ id: 50, number: '1050', accountType: 'Asset' });
		// Même de l'autre côté pour le type du formulaire : un actif non postable
		// retypé au passif dans le formulaire ne se propose pas à lui-même.
		expect(ids(mirrorCandidates(lui, { accountType: 'Liability' }, [lui, ...PLAN]))).toEqual([ACTIF.id]);
	});

	it('suit le type CHOISI dans le formulaire, non celui en base', () => {
		expect(ids(mirrorCandidates(SOURCE, { accountType: 'Expense' }, PLAN))).toEqual([PRODUIT.id]);
	});

	it('garde le miroir enregistré, en tête, même s’il ne passe plus le filtre', () => {
		const lie = { ...SOURCE, mirrorAccountId: ACTIF_IMPUTABLE.id };
		const rendu = ids(mirrorCandidates(lie, { accountType: 'Liability' }, PLAN));
		expect(rendu[0]).toBe(ACTIF_IMPUTABLE.id);
		expect(rendu).toContain(ACTIF.id);
	});

	it('à la création (compte null) : les candidats du type choisi', () => {
		expect(ids(mirrorCandidates(null, { accountType: 'Liability' }, PLAN))).toEqual([ACTIF.id]);
	});

	it('écarte un compte de groupe (qui a des sous-comptes), archivés compris (M-1)', () => {
		const groupe = acc({ id: 70, number: '1049', accountType: 'Asset' });
		const enfant = acc({ id: 71, number: '1049.1', accountType: 'Asset', parentId: 70, active: false });
		const rendu = ids(mirrorCandidates(SOURCE, { accountType: 'Liability' }, [...PLAN, groupe, enfant]));
		expect(rendu).toEqual([ACTIF.id]);
		// Le compte modifié, seul sous-compte du candidat, ne le disqualifie pas.
		const sous = { ...SOURCE, parentId: 70 };
		expect(ids(mirrorCandidates(sous, { accountType: 'Liability' }, [sous, ACTIF, groupe]))).toEqual([
			ACTIF.id,
			groupe.id,
		]);
	});

	it('garde l’ordre du plan', () => {
		const autre = acc({ id: 60, number: '1090', accountType: 'Asset' });
		expect(ids(mirrorCandidates(SOURCE, { accountType: 'Liability' }, [...PLAN, autre]))).toEqual([
			ACTIF.id,
			autre.id,
		]);
	});
});

describe('mirrorSourceNumbers', () => {
	it('les numéros des comptes du plan qui désignent le compte', () => {
		const a = { ...SOURCE, mirrorAccountId: ACTIF.id };
		const b = acc({ id: 70, number: '2271', accountType: 'Liability', mirrorAccountId: ACTIF.id });
		expect(mirrorSourceNumbers(ACTIF.id, [a, ACTIF, b])).toEqual(['2270', '2271']);
		expect(mirrorSourceNumbers(PASSIF.id, [a, ACTIF, b])).toEqual([]);
	});
});

describe('champs envoyés', () => {
	it('création : absent pour « Aucun », l’identifiant sinon', () => {
		expect(mirrorFieldForCreate('')).toEqual({});
		expect(mirrorFieldForCreate('12')).toEqual({ mirrorAccountId: 12 });
	});

	it('modification : absent s’il n’a pas changé', () => {
		expect(mirrorFieldForUpdate({}, '')).toEqual({});
		expect(mirrorFieldForUpdate({ mirrorAccountId: 12 }, '12')).toEqual({});
	});

	it('modification : null pour « Aucun », l’identifiant pour un autre miroir', () => {
		expect(mirrorFieldForUpdate({ mirrorAccountId: 12 }, '')).toEqual({ mirrorAccountId: null });
		expect(mirrorFieldForUpdate({ mirrorAccountId: 12 }, '13')).toEqual({ mirrorAccountId: 13 });
		expect(mirrorFieldForUpdate({}, '13')).toEqual({ mirrorAccountId: 13 });
	});
});

describe('mirrorFieldErrorOf', () => {
	it('le message du serveur pour MIRROR_ACCOUNT_INVALID, null sinon', () => {
		const err = { code: 'MIRROR_ACCOUNT_INVALID', message: 'Refus 1179', status: 400 };
		expect(mirrorFieldErrorOf(err)).toBe('Refus 1179');
		expect(mirrorFieldErrorOf({ ...err, code: 'ACCOUNT_IS_MIRROR', status: 409 })).toBeNull();
		expect(mirrorFieldErrorOf(new Error('x'))).toBeNull();
	});
});
