<script lang="ts">
  import { ShieldAlert, Ambulance, MapPin, Menu, Flame, Terminal } from "lucide-svelte";

  export let logs: string[];
  export let onHandleSos: (type: string, desc: string) => void;
</script>

<div class="max-w-[800px] mx-auto flex flex-col gap-8">
  <div class="text-center mt-6">
    <div class="flex justify-center">
        <ShieldAlert size={48} color="#e74c3c" />
    </div>
    <h1 class="text-[#e74c3c] my-2 text-2xl tracking-widest font-bold">EMERGENCY BROADCAST</h1>
    <p class="text-muted-foreground">Black Box Protocol active. Your location and identity are encrypted.</p>
  </div>

  <div class="grid grid-cols-2 gap-4">
    <button class="bg-card border rounded-lg p-8 flex flex-col items-center justify-center cursor-pointer transition-all hover:-translate-y-0.5 active:translate-y-px border-destructive text-destructive hover:bg-destructive/10" onclick={() => onHandleSos('Medical', 'Medical Assistance Required')}>
      <Ambulance size={40} />
      <h3 class="my-4 text-lg font-bold">MEDICAL</h3>
      <p class="text-xs text-muted-foreground">Injury, Bleeding, Sick</p>
    </button>

    <button class="bg-card border rounded-lg p-8 flex flex-col items-center justify-center cursor-pointer transition-all hover:-translate-y-0.5 active:translate-y-px border-orange-500 text-orange-500 hover:bg-orange-500/10" onclick={() => onHandleSos('Evacuation', 'Evacuation Required')}>
      <MapPin size={40} />
      <h3 class="my-4 text-lg font-bold">EVACUATION</h3>
      <p class="text-xs text-muted-foreground">Trapped, Transport needed</p>
    </button>

    <button class="bg-card border rounded-lg p-8 flex flex-col items-center justify-center cursor-pointer transition-all hover:-translate-y-0.5 active:translate-y-px border-blue-500 text-blue-500 hover:bg-blue-500/10" onclick={() => onHandleSos('FoodWater', 'Supplies Required')}>
      <Menu size={40} />
      <h3 class="my-4 text-lg font-bold">SUPPLIES</h3>
      <p class="text-xs text-muted-foreground">Food, Water, Meds</p>
    </button>

    <button class="bg-card border rounded-lg p-8 flex flex-col items-center justify-center cursor-pointer transition-all hover:-translate-y-0.5 active:translate-y-px border-purple-500 text-purple-500 hover:bg-purple-500/10" onclick={() => onHandleSos('Violence', 'Under Attack')}>
      <Flame size={40} />
      <h3 class="my-4 text-lg font-bold">DANGER</h3>
      <p class="text-xs text-muted-foreground">Violence, Shelling</p>
    </button>
  </div>

  <div class="bg-black border border-border rounded-md mt-6 flex flex-col font-mono text-xs">
    <div class="bg-muted px-3 py-1 text-muted-foreground text-[10px] flex items-center gap-2 border-b border-border">
      <Terminal size={14} />
      <span>SECURE LOG</span>
    </div>
    <div class="p-3 h-36 overflow-y-auto text-green-500">
      {#each logs as log, i (i)}
        <div class="mb-1 border-b border-white/10 pb-1">{log}</div>
      {/each}
      {#if logs.length === 0}
        <div class="opacity-50">System ready. Waiting for input...</div>
      {/if}
    </div>
  </div>
</div>
