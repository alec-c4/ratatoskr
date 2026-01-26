<script lang="ts">
  import { Radio, LogIn } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";

  export let accountCreated: boolean;
  export let onLoadIdentity: () => void;
  export let onGetStarted: () => void;
</script>

<div class="welcome-overlay">
  <Radio size={64} color="#27ae60" />
  <h2 class="text-2xl font-bold mt-4 mb-2">Welcome to Ratatoskr</h2>
  
  {#if accountCreated}
      <p class="text-muted-foreground mb-6">Identity found on this device.</p>
      <Button onclick={onLoadIdentity} class="gap-2">
          <LogIn size={18} />
          Log In
      </Button>
  {:else}
      <p class="text-muted-foreground mb-6">To start communicating securely, you need to create an identity.</p>
      <Button onclick={onGetStarted}>Get Started</Button>
  {/if}
</div>

<style>
  .welcome-overlay {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    animation: fadeIn 0.5s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
