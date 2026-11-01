// Lumière des blocs (torches, lanternes, lave, glowstone…) : niveaux 0 à 15 propagés
// bloc par bloc au chargement (bs_monde::lumiere), rangés dans une texture 3D lissée.
// On lit le niveau de l'air juste devant la surface, puis la courbe de luminosité de
// Minecraft (un bloc à 8 éclaire bien moins que la moitié d'un bloc à 15) ; la lumière
// est chaude, comme celle d'une flamme.

#define_import_path bs_rendu::lumiere

const CHAUDE: vec3<f32> = vec3(1.0, 0.72, 0.42);
const INTENSITE: f32 = 2.2;

// `volume` : coin de la boîte dans le monde, active (1) ; `taille` : taille en blocs.
fn lumiere_blocs(
    niveaux: texture_3d<f32>,
    echantillonneur: sampler,
    volume: vec4<f32>,
    taille: vec4<f32>,
    monde: vec3<f32>,
    normale: vec3<f32>,
) -> vec3<f32> {
    if volume.w < 0.5 {
        return vec3(0.0);
    }
    let q = monde + normale * 0.5;
    let uvw = (q - volume.xyz) / taille.xyz;
    let v = textureSampleLevel(niveaux, echantillonneur, uvw, 0.0).r;
    let b = v / (4.0 - 3.0 * v);
    return CHAUDE * (b * INTENSITE);
}
