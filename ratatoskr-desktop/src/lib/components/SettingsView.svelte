<script lang="ts">
  import { 
    Fingerprint, 
    UserPlus, 
    Download, 
    Settings, 
    ShieldAlert, 
    Trash2, 
    LogOut 
  } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  export let userIdentity: string | null;
  export let generatedMnemonic: string;
  export let copyFeedback: string;
  export let registrationName: string;
  export let showRecovery: boolean;
  export let recoveryPhrase: string;

  export let onRegister: () => void;
  export let onRecover: () => void;
  export let onLogout: () => void;
  export let onPanicWipe: () => void;
  export let onExportKey: () => void;
  export let onDownloadBackup: () => void;
  export let onCopyMnemonic: () => void;
</script>

<div class="max-w-[600px] mx-auto flex flex-col gap-8">
  <h2 class="text-2xl font-bold">SETTINGS</h2>
  
  <div class="bg-card border rounded-lg p-6">
    <div class="flex items-center gap-2 mb-6 border-b pb-2 text-primary font-bold uppercase text-sm">
      <Fingerprint size={20} />
      <span>Identity (DID)</span>
    </div>
    
    <div class="bg-muted/30 rounded-lg p-4">
      {#if userIdentity}
        <div class="flex flex-col gap-2">
          <span class="text-xs uppercase text-muted-foreground">Public Key (Hex)</span>
          <div class="bg-black p-3 rounded font-mono text-xs text-white break-all border border-border">
            {userIdentity}
          </div>
          <p class="text-xs text-muted-foreground">This is your unique decentralized ID on the network.</p>
          
          <Button variant="outline" onclick={onExportKey} class="w-full mt-2 gap-2">
            <Download size={14} /> Export Public Key to File
          </Button>

          {#if generatedMnemonic}
            <div class="bg-yellow-50 border border-yellow-200 p-4 rounded-lg mt-4 text-yellow-900">
              <strong class="block mb-2">⚠️ SECRET RECOVERY PHRASE</strong>
              <p class="text-sm mb-2">Write this down immediately. It will not be shown again.</p>
              <button class="w-full bg-black p-4 font-mono text-sm text-white border border-dashed border-yellow-600/50 text-left hover:bg-black/80" onclick={onCopyMnemonic} type="button">
                {generatedMnemonic}
              </button>
              <div class="flex justify-between items-center mt-2">
                <Button variant="secondary" size="sm" onclick={onDownloadBackup} class="gap-2 h-8 text-xs">
                    <Download size={14} /> Save Backup File
                </Button>
                <span class="text-xs text-primary">{copyFeedback}</span>
              </div>
            </div>
          {/if}
        </div>
      {:else}
        {#if !showRecovery}
          <div class="text-center py-6">
            <p class="mb-4 text-muted-foreground">You don't have an identity yet.</p>
            <div class="flex flex-col items-center gap-4">
                <Input type="text" placeholder="Choose a Nickname" bind:value={registrationName} class="text-center text-lg" />
                <div class="flex gap-4">
                    <Button onclick={onRegister} class="gap-2">
                        <UserPlus size={18} /> Create New Account
                    </Button>
                </div>
            </div>
            <Button variant="ghost" onclick={() => showRecovery = true} class="mt-6 gap-2 text-muted-foreground">
                <Fingerprint size={18} /> I have a Recovery Phrase
            </Button>
          </div>
        {:else}
          <div class="flex flex-col gap-4">
            <h3 class="font-bold">Recover Account</h3>
            <p class="text-sm text-muted-foreground">Enter your 12-word secret phrase below:</p>
            <textarea 
              bind:value={recoveryPhrase} 
              placeholder="witch collapse practice feed shame open despair creek road again ice least"
              rows="3"
              class="w-full bg-black border rounded-md p-3 text-white font-mono resize-none focus:outline-none focus:ring-2 focus:ring-ring"
            ></textarea>
            <div class="flex justify-center gap-4 mt-2">
              <Button onclick={onRecover}>Recover</Button>
              <Button variant="ghost" onclick={() => showRecovery = false}>Cancel</Button>
            </div>
          </div>
        {/if}
      {/if}
    </div>
  </div>

  <div class="bg-card border rounded-lg p-6">
     <div class="flex items-center gap-2 mb-6 border-b pb-2 text-primary font-bold uppercase text-sm">
      <Settings size={20} />
      <span>App Preferences</span>
    </div>
    <p class="opacity-50 text-sm">More settings coming soon...</p>
    {#if userIdentity}
        <Button variant="outline" class="w-full mt-4 gap-2" onclick={onLogout}>
            <LogOut size={16} /> Log Out
        </Button>
    {/if}
  </div>

  <div class="bg-red-950/10 border border-red-900/30 rounded-lg p-6">
     <div class="flex items-center gap-2 mb-6 border-b border-red-900/30 pb-2 text-destructive font-bold uppercase text-sm">
      <ShieldAlert size={20} />
      <span>Danger Zone</span>
    </div>
    <p class="text-sm text-muted-foreground mb-4">Destructive actions. Use with extreme caution.</p>
    <Button variant="destructive" class="w-full gap-2" onclick={onPanicWipe}>
        <Trash2 size={18} /> Panic Wipe: Destroy Identity
    </Button>
  </div>
</div>
