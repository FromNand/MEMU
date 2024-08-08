unsigned int ppu_cycle;

void ppu_tick(unsigned int cycle) {
    ppu_cycle += 3 * cycle;
}

void render(void) {

}
