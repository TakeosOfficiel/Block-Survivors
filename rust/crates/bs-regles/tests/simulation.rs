//! Tests de la simulation : déterminisme (même graine, même état), vraie partie (l'arme
//! tue, les zombies frappent), et aucune allocation pendant la partie (allocateur compteur).

#![allow(unsafe_code)] // allocateur compteur : test uniquement

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use bs_regles::{Commande, Parametres, Simulation};

struct Compteur;
thread_local! {
    /// Compté par fil : les autres tests tournent en parallèle.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for Compteur {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCATIONS.with(|n| n.set(n.get() + 1));
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
}

#[global_allocator]
static ALLOCATEUR: Compteur = Compteur;

/// Le joueur tourne en rond (commande scriptée, déterministe).
fn commande(pas: u64) -> Commande {
    let a = pas as f32 * 0.01;
    Commande {
        dx: libm::cosf(a),
        dz: libm::sinf(a),
    }
}

fn jouer(graine: u64, pas: u64) -> Simulation {
    let mut s = Simulation::nouvelle(graine, Parametres::default());
    for k in 0..pas {
        s.avancer(commande(k));
    }
    s
}

#[test]
fn deterministe() {
    let (a, b) = (jouer(7, 1200), jouer(7, 1200));
    assert_eq!(a.empreinte(), b.empreinte());
    assert_ne!(a.empreinte(), jouer(8, 1200).empreinte());
}

#[test]
fn une_vraie_partie() {
    let s = jouer(1, 60 * 60);
    assert!(s.tues > 100, "l'arme doit tuer : {}", s.tues);
    assert!(s.degats_subis > 0.0, "les zombies doivent frapper");
}

#[test]
fn aucune_allocation_en_partie() {
    let mut s = Simulation::nouvelle(3, Parametres::default());
    s.avancer(Commande::default());
    let avant = ALLOCATIONS.with(Cell::get);
    for k in 0..600 {
        s.avancer(commande(k));
    }
    assert_eq!(ALLOCATIONS.with(Cell::get), avant);
}
