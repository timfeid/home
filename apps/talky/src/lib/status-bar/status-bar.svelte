<script lang="ts">
	import { Mic, MicOff, Volume2, VolumeX } from 'lucide-svelte';

	let muted = $state(false);
	let deafened = $state(false);
</script>

<div class="flex items-center border-t border-gray-400 bg-gray-300 p-1 text-xs">
	<div class="flex items-center">
		<span class="mr-1 font-bold">Status:</span>
		<span class="text-green-700">Connected</span>
	</div>
	<div class="mx-2 text-gray-500">|</div>
	<div>
		<span class="mr-1 font-bold">Lag:</span>
		<span>54ms</span>
	</div>
	<div class="mx-2 text-gray-500">|</div>
	<div>
		<span class="mr-1 font-bold">Users:</span>
		<span>5</span>
	</div>
	<div class="mx-2 text-gray-500">|</div>
	<div>
		<span class="mr-1 font-bold">Channels:</span>
		<span>6</span>
	</div>

	<div class="ml-auto flex items-center">
		<button
			onclick={() => (muted = !muted)}
			class="mr-1 p-1 {muted ? 'bg-red-200 text-red-700' : 'bg-gray-200'} border border-gray-400"
			title={muted ? 'Unmute' : 'Mute'}
		>
			{#if muted}
				<MicOff class="h-3 w-3" />
			{:else}
				<Mic class="h-3 w-3" />
			{/if}
		</button>
		<button
			onclick={() => (deafened = !deafened)}
			class="p-1 {deafened ? 'bg-red-200 text-red-700' : 'bg-gray-200'} border border-gray-400"
			title={deafened ? 'Undeafen' : 'Deafen'}
		>
			{#if deafened}
				<VolumeX class="h-3 w-3" />
			{:else}
				<Volume2 class="h-3 w-3" />
			{/if}
		</button>
	</div>
</div>
