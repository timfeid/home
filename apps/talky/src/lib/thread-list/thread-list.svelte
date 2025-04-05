<script lang="ts">
	import { Hash, Send } from 'lucide-svelte';

	let message = $state('');

	// Mock message data
	const messages = [
		{ id: 1, time: '14:30', nick: 'Server', content: '*** Now talking in #gaming', system: true },
		{ id: 2, time: '14:31', nick: 'Admin', content: 'Welcome to the gaming channel!', op: true },
		{
			id: 3,
			time: '14:32',
			nick: 'GamerGirl',
			content: "Hey everyone! How's it going?",
			voice: true
		},
		{
			id: 4,
			time: '14:33',
			nick: 'CoolDude',
			content: 'Just finished a great match!',
			normal: true
		},
		{
			id: 5,
			time: '14:35',
			nick: 'NightOwl',
			content: 'Anyone up for some games tonight?',
			normal: true
		},
		{ id: 6, time: '14:36', nick: 'Admin', content: "I'll set up the server at 9pm", op: true },
		{ id: 7, time: '14:37', nick: 'GamerGirl', content: "Perfect! I'll be there", voice: true },
		{ id: 8, time: '14:38', nick: 'Server', content: '*** NightOwl is now away: AFK', system: true }
	];

	function handleSubmit(e: Event) {
		e.preventDefault();
		// Handle message submission
		message = '';
	}
</script>

<div class="flex items-center bg-blue-800 p-1 text-white">
	<Hash class="mr-1 h-4 w-4" />
	<span class="font-bold">#gaming</span>
	<span class="ml-2 text-xs">(+nt)</span>
	<span class="ml-auto text-xs">Topic: Welcome to the Gaming channel! | Rules: Be nice</span>
</div>
<div class="flex-1 overflow-y-auto bg-white p-1 font-mono text-sm">
	{#each messages as msg (msg.id)}
		<div class="leading-5 hover:bg-gray-100">
			{#if msg.system}
				<div class="text-blue-600">
					{msg.time}
					{msg.content}
				</div>
			{:else}
				<div>
					<span class="text-gray-500">[{msg.time}] </span>
					<span
						class="font-bold {msg.op
							? 'text-red-600'
							: msg.voice
								? 'text-blue-600'
								: 'text-gray-800'}"
					>
						&lt;{msg.nick}&gt;
					</span>
					<span> {msg.content}</span>
				</div>
			{/if}
		</div>
	{/each}
</div>
<div class="border-t border-t-neutral-900 bg-gray-200 p-1">
	<form onsubmit={handleSubmit} class="flex items-center">
		<input
			type="text"
			bind:value={message}
			placeholder="Type a message or command..."
			class="flex-1 border px-2 py-1 font-mono text-sm"
		/>
		<button type="submit" class="border border-l-0 bg-gray-300 px-2 py-1">
			<Send class="h-4 w-4" />
		</button>
	</form>
</div>
