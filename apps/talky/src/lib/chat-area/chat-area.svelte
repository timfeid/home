<script lang="ts">
	import { Avatar, AvatarFallback } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { withPresence, type ChatMessage } from '$lib/presence.svelte';
	import UserList from '$lib/user-list/user-list.svelte';
	import type { Procedures } from '@feid/bindings';
	import { AtSign, Paperclip, Send, Smile } from 'lucide-svelte';
	import { onMount } from 'svelte';

	let { channel }: { channel: Procedures['niche_find_by_slug']['output']; slug: string } = $props();

	let input = $state('');
	const presence = withPresence();

	async function chat(event: Event) {
		event.preventDefault();
		if (input) {
			const content = $state.snapshot(input);
			input = '';
			try {
				await presence.sendMessage({
					type: 'chat_message',
					content,
					channel_id: channel.id
				});
			} catch (e) {
				console.error(e);
				input = content;
			}
		}
	}

	onMount(() => {
		presence.on('chatMessageReceived', (chat: ChatMessage) => {
			if (chat.channel_id === channel.id) {
				messages.push(chat);
			}
		});
	});

	const messages: ChatMessage[] = $state([]);
</script>

<div class="flex h-full w-full">
	<div class="flex h-full flex-1 flex-col overflow-y-auto">
		<div class="mt-auto space-y-4 p-4">
			{#each messages as message}
				<div class={message.isSystem ? 'text-center' : ''}>
					{#if message.isSystem}
						<div class="inline-block rounded-md bg-primary px-3 py-1 text-primary-foreground">
							<p class="font-mono text-xs">{message.content}</p>
						</div>
					{:else}
						<div class="flex items-start">
							<Avatar class="mr-2 mt-1 h-8 w-8">
								<!-- <AvatarImage src={message.a} alt={message.author.name} /> -->
								<AvatarFallback class="bg-foreground text-xs text-background">
									{message.user_id.substring(0, 2).toUpperCase()}
								</AvatarFallback>
							</Avatar>
							<div class="flex-1">
								<div class="flex items-baseline">
									<span class="font-mono font-bold text-foreground/60">{message.user_id}</span>
									<span class="ml-2 font-mono">{message.timestamp}</span>
								</div>
								<p class="font-mono text-sm">{message.content}</p>
							</div>
						</div>
					{/if}
				</div>
			{/each}
		</div>

		<form onsubmit={chat} class="p-2">
			<div class="rounded bg-primary/10 p-2">
				<div class=" flex items-center">
					<Button variant="ghost" size="icon" class="h-8 w-8  hover:text-primary">
						<Paperclip class="h-4 w-4" />
					</Button>
					<div class="relative mx-2 flex-1">
						<Input
							class=" border-transparent bg-transparent font-mono"
							placeholder="Message #gameday"
							bind:value={input}
						/>
						<div
							class="absolute right-2 top-1/2 flex -translate-y-1/2 transform items-center space-x-1"
						>
							<Button variant="ghost" size="icon" class="h-6 w-6  hover:text-primary">
								<AtSign class="h-4 w-4" />
							</Button>
							<Button variant="ghost" size="icon" class="h-6 w-6  hover:text-primary">
								<Smile class="h-4 w-4" />
							</Button>
						</div>
					</div>
					<Button
						type="submit"
						size="icon"
						class="h-8 w-8 bg-primary text-primary-foreground/60 hover:text-primary-foreground"
					>
						<Send class="h-4 w-4" />
					</Button>
				</div>
			</div>
		</form>
	</div>
	<div>
		<UserList {channel} />
	</div>
</div>
