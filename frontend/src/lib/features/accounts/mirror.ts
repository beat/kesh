/**
 * Story 47-2 (AC-2, AC-3) — le **compte miroir** au dialogue d'un compte.
 *
 * Un compte miroir est un compte de présentation (non postable, sans rôle ni
 * écriture), de l'autre côté du même rapport (actif ↔ passif, produit ↔
 * charge), sur lequel le solde de sens inverse d'un compte paraîtra dans le
 * bilan ou le compte de résultat (Story 47-1).
 *
 * ⚠️ **L'écran ne décide d'aucune règle** (C-47-2-5) : ces fonctions filtrent
 * les candidats sur ce que le plan chargé sait (type, postable, rôle, actif,
 * chaîne), et le serveur reste juge du reste — écritures, rôle de fonds propres
 * de la source, sources archivées non affichées. Son refus (`err.message`,
 * traduit et qui nomme le compte en cause) s'affiche sous le champ.
 */

import { isApiError } from '$lib/shared/utils/api-client';
import { withCurrentAccount } from './account-options';
import type { AccountResponse, AccountType } from './accounts.types';

/** Le type de l'autre côté du même rapport : actif ↔ passif, produit ↔ charge. */
export function oppositeAccountType(t: AccountType): AccountType {
	switch (t) {
		case 'Asset':
			return 'Liability';
		case 'Liability':
			return 'Asset';
		case 'Revenue':
			return 'Expense';
		case 'Expense':
			return 'Revenue';
	}
}

/**
 * Les comptes proposés comme miroir, dans l'ordre du plan : actifs, autres que
 * le compte lui-même, de l'autre côté du type **choisi dans le formulaire**,
 * non postables, sans rôle, sans miroir eux-mêmes, **sans sous-compte** : un
 * compte de groupe n'est pas un compte miroir (le serveur le refuse,
 * `MIRROR_ACCOUNT_INVALID` `hasChildren` — revue d'intégration P1 de l'Epic 47,
 * M-1). Le compte modifié ne compte pas parmi les sous-comptes d'un candidat
 * (le serveur juge l'état résultant).
 *
 * Le miroir **enregistré** figure toujours dans la liste, en tête, même s'il
 * ne passe plus le filtre (`withCurrentAccount`) : le champ ne s'affiche
 * jamais vide sur une valeur en base.
 *
 * @param account le compte modifié, ou `null` à la création
 * @param form le type choisi dans le formulaire
 * @param accounts le plan que la page a chargé (archivés seulement s'ils sont affichés)
 */
export function mirrorCandidates(
	account: Pick<AccountResponse, 'id' | 'mirrorAccountId'> | null,
	form: { accountType: AccountType },
	accounts: AccountResponse[],
): AccountResponse[] {
	const side = oppositeAccountType(form.accountType);
	const parents = new Set(
		accounts
			.filter((c) => c.id !== account?.id)
			.map((c) => c.parentId)
			.filter((id): id is number => id !== null),
	);
	const filtered = accounts.filter(
		(a) =>
			a.active &&
			a.id !== account?.id &&
			a.accountType === side &&
			!a.postable &&
			a.role === null &&
			(a.mirrorAccountId ?? null) === null &&
			!parents.has(a.id),
	);
	return withCurrentAccount(filtered, account?.mirrorAccountId ?? null, accounts);
}

/**
 * Les numéros des comptes **du plan chargé** qui désignent `accountId` comme
 * leur miroir, dans l'ordre du plan. Non vide : le compte est lui-même un
 * compte miroir, et le dialogue montre une phrase au lieu du `<select>`.
 */
export function mirrorSourceNumbers(accountId: number, accounts: AccountResponse[]): string[] {
	return accounts.filter((a) => a.mirrorAccountId === accountId).map((a) => a.number);
}

/**
 * Le champ `mirrorAccountId` d'une création : absent pour « Aucun » (`''`),
 * l'identifiant sinon.
 */
export function mirrorFieldForCreate(selected: string): { mirrorAccountId?: number } {
	return selected === '' ? {} : { mirrorAccountId: Number(selected) };
}

/**
 * Le champ `mirrorAccountId` d'un `PUT` : **absent** s'il n'a pas changé
 * (absent = inchangé côté serveur : un client qui ne connaît pas le champ
 * n'efface rien), `null` pour « Aucun », l'identifiant sinon.
 */
export function mirrorFieldForUpdate(
	account: Pick<AccountResponse, 'mirrorAccountId'>,
	selected: string,
): { mirrorAccountId?: number | null } {
	const mirrorAccountId = selected === '' ? null : Number(selected);
	return mirrorAccountId === (account.mirrorAccountId ?? null) ? {} : { mirrorAccountId };
}

/**
 * Le refus `MIRROR_ACCOUNT_INVALID`, à afficher sous le `<select>` : le message
 * du serveur tel quel (traduit, il nomme le compte en cause) ; `null` pour tout
 * autre refus, qui garde son chemin.
 */
export function mirrorFieldErrorOf(err: unknown): string | null {
	return isApiError(err) && err.code === 'MIRROR_ACCOUNT_INVALID' ? err.message : null;
}
