<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Mic, MicOff, Volume2, VolumeX } from 'lucide-svelte';

	let muted = $state(false);
	let deafened = $state(false);
</script>

<div class="flex items-center border-t p-1 text-xs">
	<div class="flex items-center">
		<span class="mr-1 font-bold">Status:</span>
		<span class="text-green-700">Connected</span>
	</div>
	<div class="mx-2 text-neutral-500">|</div>
	<div>
		<span class="mr-1 font-bold">Lag:</span>
		<span>54ms</span>
	</div>
	<div class="mx-2 text-neutral-500">|</div>
	<div>
		<span class="mr-1 font-bold">Users:</span>
		<span>5</span>
	</div>
	<div class="mx-2 text-neutral-500">|</div>
	<div>
		<span class="mr-1 font-bold">Channels:</span>
		<span>6</span>
	</div>

	<div class="ml-auto flex items-center">
		<Button
			size="icon"
			variant="secondary"
			onclick={() => (muted = !muted)}
			class="mr-1 h-5 w-5 {muted ? 'text-red-700' : ''}"
			title={muted ? 'Unmute' : 'Mute'}
		>
			{#if muted}
				<MicOff class="h-3 w-3" />
			{:else}
				<Mic class="h-3 w-3" />
			{/if}
		</Button>
		<Button
			size="icon"
			variant="secondary"
			onclick={() => (deafened = !deafened)}
			class="h-5 w-5 {deafened ? 'text-red-700' : ''}"
			title={deafened ? 'Undeafen' : 'Deafen'}
		>
			{#if deafened}
				<VolumeX class="h-2 w-2" />
			{:else}
				<Volume2 class="h-2 w-2" />
			{/if}
		</Button>
	</div>
</div>
