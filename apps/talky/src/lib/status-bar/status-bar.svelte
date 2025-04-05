<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { withPresence } from '$lib/presence.svelte';
	import { Mic, MicOff, Volume2, VolumeX } from 'lucide-svelte';

	let muted = $state(false);
	let deafened = $state(false);
</script>

<div class="flex items-center border-t px-3 py-1 text-[10px]">
	<div class="flex items-center">
		<span class="mr-1 font-bold">Status:</span>
		<span class="text-green-700">Connected</span>
	</div>
	<div class="mx-2 text-neutral-500">|</div>
	<!-- <div>
		<span class="mr-1 font-bold">Lag:</span>
		<span>54ms</span>
	</div>
	<div class="mx-2 text-neutral-500">|</div>-->
	<div>
		<span class="mr-1 font-bold">Users:</span>
		<span>{withPresence().activeClients.length}</span>
	</div>

	<div class="ml-auto flex items-center">
		<Button
			size="icon"
			variant="secondary"
			onclick={() => (muted = !muted)}
			class="mr-1 h-4 w-4 bg-transparent p-3 {muted ? 'text-red-700' : ''}"
			title={muted ? 'Unmute' : 'Mute'}
		>
			{#if muted}
				<MicOff class="size-3" />
			{:else}
				<Mic class="size-3" />
			{/if}
		</Button>
		<Button
			size="icon"
			variant="secondary"
			onclick={() => (deafened = !deafened)}
			class="mr-1 h-4 w-4 bg-transparent p-3 {deafened ? 'text-red-700' : ''}"
			title={deafened ? 'Undeafen' : 'Deafen'}
		>
			{#if deafened}
				<VolumeX class="size-3" />
			{:else}
				<Volume2 class="size-3" />
			{/if}
		</Button>
	</div>
</div>
