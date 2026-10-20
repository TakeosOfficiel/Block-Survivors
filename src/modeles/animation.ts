/**
 * Animation des modèles cubiques (balancement des membres façon Minecraft).
 * L'état d'animation est un petit objet par entité ; le calcul écrit les
 * rotations de chaque partie dans un tampon, sans allocation.
 */
import { facteurLissage } from '../utils/maths';
import type { ModeleCompile } from './compilation';

export interface EtatAnimation {
  phase: number;      // cycle de marche (rad)
  intensite: number;  // 0 = immobile, 1 = pleine allure
  temps: number;      // horloge du balancement au repos (s)
  attaque: number;    // s restantes du geste d'attaque (0 = aucun)
}

export function creerEtatAnimation(): EtatAnimation {
  return { phase: 0, intensite: 0, temps: 0, attaque: 0 };
}

export function reinitialiserAnimation(etat: EtatAnimation): void {
  etat.phase = 0;
  etat.intensite = 0;
  etat.temps = Math.random() * 10;  // désynchronise le balancement des entités
  etat.attaque = 0;
}

/** Lance le geste d'attaque du modèle (sans effet s'il n'en a pas). */
export function declencherAttaque(etat: EtatAnimation, modele: ModeleCompile): void {
  etat.attaque = modele.definition.attaque?.duree ?? 0;
}

const TOUR = Math.PI * 2;

/** Avance l'animation : dt en temps de jeu (s), vitesseSol en blocs/s. */
export function avancerAnimation(etat: EtatAnimation, modele: ModeleCompile, dt: number, vitesseSol: number): void {
  etat.temps += dt;
  if (etat.attaque > 0) etat.attaque = Math.max(0, etat.attaque - dt);
  const marche = modele.definition.marche;
  if (!marche) return;
  etat.phase = (etat.phase + vitesseSol * dt * marche.frequence) % TOUR;
  const cible = Math.min(vitesseSol / marche.vitesseReference, 1);
  etat.intensite += (cible - etat.intensite) * facteurLissage(marche.lissage, dt);
}

/** Écrit dans `sortie` les rotations (x, y, z) de chaque partie : pose de base + animation. */
export function calculerRotations(modele: ModeleCompile, etat: EtatAnimation, sortie: Float32Array): void {
  sortie.set(modele.poseBase);

  for (let i = 0; i < modele.marche.length; i++) {
    const m = modele.marche[i];
    const angle = etat.phase * m.frequence + m.dephasage;
    const onde = m.absolu ? Math.abs(Math.sin(angle)) : Math.cos(angle);
    sortie[m.partie * 3 + m.axe] += onde * m.amplitude * etat.intensite;
  }

  const repos = modele.definition.repos;
  if (repos) {
    const ecart = Math.cos(etat.temps * repos.frequenceZ) * repos.amplitude + repos.amplitude;
    const avant = Math.sin(etat.temps * repos.frequenceX) * repos.amplitude;
    for (let i = 0; i < modele.repos.length; i++) {
      const m = modele.repos[i];
      sortie[m.partie * 3] += m.signe * avant;
      sortie[m.partie * 3 + 2] -= m.signe * ecart;  // écarte le bras du corps
    }
  }

  const attaque = modele.definition.attaque;
  if (attaque && etat.attaque > 0) {
    const levee = Math.sin((1 - etat.attaque / attaque.duree) * Math.PI);
    for (let i = 0; i < modele.attaque.length; i++) {
      const m = modele.attaque[i];
      sortie[m.partie * 3] -= levee * m.amplitude;
    }
  }
}

/** Angle de chute sur le côté après la mort (courbe du jeu : rapide puis amortie). */
export function angleChute(temps: number, duree: number): number {
  return Math.min(1, Math.sqrt((temps / duree) * 1.6)) * (Math.PI / 2);
}
