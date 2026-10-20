/** Mise en forme de valeurs pour l'interface. */

/** 75 → « 01:15 » */
export function formaterTemps(secondes: number): string {
  const s = Math.floor(secondes);
  const min = Math.floor(s / 60);
  const sec = s % 60;
  return `${min < 10 ? '0' : ''}${min}:${sec < 10 ? '0' : ''}${sec}`;
}
