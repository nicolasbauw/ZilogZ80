Pour que l'Amstrad CPC puisse démarrer et exécuter son OS (puis le BASIC), la gestion des interruptions du Z80 doit être d'une précision chirurgicale. Une seule approximation sur les registres IFF1/IFF2 ou sur le timing d'une instruction comme EI empêchera la ROM de s'initialiser.

Voici l'audit complet des points critiques à vérifier dans le code de votre CPU Z80 (src/cpu.rs), basé sur les implémentations standards et les pièges classiques de l'émulation Z80.

1. Audit des instructions d'interruptions
DI (Disable Interrupts)
L'instruction DI désactive les interruptions masquables.

Comportement attendu : Elle doit mettre à false (ou 0) les deux bascules d'interruption : IFF1 = 0 et IFF2 = 0.

Timing : L'effet est immédiat.

Dans votre code, vérifiez que :

Rust
// Exemple de ce que doit faire l'opcode 0xF3 (DI)
self.iff1 = false;
self.iff2 = false;
EI (Enable Interrupts) — Le piège classique !
L'instruction EI réactive les interruptions masquables.

Comportement attendu : Elle doit mettre IFF1 = true et IFF2 = true.

Le piège de l'effet retardé : Sur un vrai Z80, l'activation des interruptions ne prend effet qu'après l'instruction qui suit immédiatement le EI. Cela permet à une instruction de retour (comme RET ou RETI) de s'exécuter avant qu'une nouvelle interruption ne vienne interrompre le CPU.

Dans votre code, vérifiez que :
Si vous activez iff1 immédiatement dans l'exécution de l'opcode 0xFB (EI), le CPU risque d'accepter une interruption pendant la séquence critique de retour de l'OS. Vous devez implémenter un mécanisme de délai (par exemple, un flag ei_delay ou interrupt_delay qui empêche la prise en compte d'une interruption pendant exactement un cycle d'instruction).

Rust
// Exemple d'implémentation propre de l'opcode 0xFB (EI)
self.iff1 = true;
self.iff2 = true;
self.ei_delay = true; // Empêche l'interruption de se déclencher AUJOURD'HUI
LD A, I et LD A, R
Ces instructions transfèrent le registre d'interruption I ou de rafraîchissement R dans l'accumulateur A.

Comportement attendu : En plus du transfert, ces deux instructions ont une fonction vitale : elles copient l'état de IFF2 dans le Flag de Parité/Débordement (P/V) du registre des flags (F). C'est le seul moyen pour un programme de savoir si les interruptions étaient activées avant qu'il ne les modifie.

Dans votre code, vérifiez que :

Rust
// Exemple pour LD A, I (Opcode ED 57)
self.a = self.i;
self.f.sign = (self.a & 0x80) != 0;
self.f.zero = self.a == 0;
self.f.half_carry = false;
self.f.pv = self.iff2; // CRUCIAL : IFF2 est copié dans le flag P/V
self.f.add_subtract = false;
2. Gestion des INT / NMI et des bascules IFF1 / IFF2
Le rôle de IFF1 et IFF2
IFF1 (Interrupt Flip-Flop 1) : Détermine si le CPU accepte actuellement les interruptions masquables (INT).

IFF2 (Interrupt Flip-Flop 2) : Sert de zone de sauvegarde temporaire pour IFF1 lorsqu'une interruption non masquable (NMI) survient. Cela permet de restaurer l'état initial des interruptions à la fin de la routine de la NMI (via l'instruction RETN).

Gestion d'une interruption standard (INT)
L'Amstrad CPC utilise presque exclusivement le Mode d'interruption 1 (IM 1). Lorsqu'une ligne d'interruption est tirée vers le bas par le Gate Array et que IFF1 est actif :

Désactivation des interruptions : Le CPU met automatiquement IFF1 = false et IFF2 = false pour éviter que les interruptions ne s'empilent à l'infini.

Sauvegarde : Le CPU pousse le pointeur de programme actuel (PC) sur la pile.

Saut : Le CPU saute directement à l'adresse fixe $0038.

Dans votre code, vérifiez la transition d'état :

Rust
if self.iff1 && !self.ei_delay && self.int_pending {
    self.iff1 = false;
    self.iff2 = false;
    self.push_u16(bus, self.pc);
    self.pc = 0x0038; // En mode IM 1 (spécifique au CPC)
}
Gestion d'une interruption non masquable (NMI)
Même si le CPC classique n'utilise pas activement la ligne NMI, votre émulateur Z80 doit l'implémenter correctement :

Sauvegarde de l'état : L'état de IFF1 est copié dans IFF2.

Désactivation : Le CPU met IFF1 = false (pour éviter qu'une interruption standard INT ne vienne perturber la routine NMI). Attention : IFF2 conserve l'ancienne valeur de IFF1.

Saut : Le CPU pousse PC sur la pile et saute à l'adresse fixe $0066.

Dans votre code, vérifiez la transition d'état :

Rust
if self.nmi_pending {
    self.nmi_pending = false;
    self.iff2 = self.iff1; // Sauvegarde de l'état
    self.iff1 = false;     // On coupe les INT standards
    self.push_u16(bus, self.pc);
    self.pc = 0x0066;
}
L'instruction RETN (Return from NMI)
Comportement attendu : Cette instruction termine une routine NMI. Elle doit impérativement restaurer l'état des interruptions en recopiant IFF2 dans IFF1.

Dans votre code, vérifiez que :

Rust
self.pc = self.pop_u16(bus);
self.iff1 = self.iff2; // Restauration automatique
