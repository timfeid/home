<script lang="ts">
  import { Mic, MicOff, Volume2, VolumeX, Phone } from 'lucide-svelte';

  let muted = $state(false);
  let deafened = $state(false);
  let connected = $state(true);
</script>

<div class="flex items-center border-t bg-gray-300 p-1">
  <div class="text-xs">
    <span class="font-bold">Voice:</span>
    <span class={connected ? 'ml-1 text-green-700' : 'ml-1 text-red-700'}>
      {connected ? 'Connected to #lobby' : 'Disconnected'}
    </span>
  </div>

  <div class="ml-auto flex items-center">
    <button
      onclick={() => (muted = !muted)}
      class="mr-1 p-1 {muted ? 'bg-red-200 text-red-700' : 'bg-gray-200'} border text-xs"
    >
      {#if muted}
        <MicOff class="h-3 w-3" />
      {:else}
        <Mic class="h-3 w-3" />
      {/if}
      <span class="ml-1">{muted ? 'Unmute' : 'Mute'}</span>
    </button>
    <button
      onclick={() => (deafened = !deafened)}
      class="mr-1 p-1 {deafened ? 'bg-red-200 text-red-700' : 'bg-gray-200'} border text-xs"
    >
      {#if deafened}
        <VolumeX class="h-3 w-3" />
      {:else}
        <Volume2 class="h-3 w-3" />
      {/if}
      <span class="ml-1">{deafened ? 'Undeafen' : 'Deafen'}</span>
    </button>
    <button onclick={() => (connected = !connected)} class="border bg-gray-200 p-1 text-xs">
      <Phone class="h-3 w-3" />
      <span class="ml-1">{connected ? 'Disconnect' : 'Connect'}</span>
    </button>
  </div>
</div>
