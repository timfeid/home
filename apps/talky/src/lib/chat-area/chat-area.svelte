<script lang="ts">
	import { client, wrapResponse } from '$lib/client';
	import { Avatar, AvatarFallback } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { withPresence, type ChatMessage } from '$lib/presence.svelte';
	import type { Procedures } from '@feid/bindings';
	import { AtSign, Paperclip, Send, Smile } from 'lucide-svelte';
	import { onMount } from 'svelte';

	let { channel }: { channel: Procedures['niche_find_by_slug']['output']; slug: string } = $props();

	let input = $state('');
	const presence = withPresence();

	async function chat(event: Event) {
		event.preventDefault();
		if (input) {
			presence.sendMessage({
				type: 'chat_message',
				content: $state.snapshot(input),
				channel_id: channel.id
			});
			input = '';
		}
	}

	async function getChat() {
		try {
			const chat = wrapResponse(await client.channel_messages.query({ channel_id: channel.id }));
			messages.unshift(...chat.edges.map((edge) => edge.node));
		} catch (e) {
			console.error(e);
		}
	}

	onMount(async () => {
		await getChat();
		presence.on('chatMessageReceived', (chat: ChatMessage) => {
			if (chat.channel_id === channel.id) {
				messages.push(chat.message);
			}
		});
	});

	const messages: Procedures['channel_messages']['output']['edges'][number]['node'][] = $state([]);

	function timeAgo(timestamp: number) {
		const currentDate = new Date();
		const currentTimestamp = currentDate.getTime();
		const messageDate = new Date(timestamp);
		const differenceInMilliseconds = currentTimestamp - timestamp;

		const millisecondsInADay = 24 * 60 * 60 * 1000;

		if (differenceInMilliseconds < millisecondsInADay) {
			return messageDate.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
		}

		if (differenceInMilliseconds < 2 * millisecondsInADay) {
			return `yesterday @ ${messageDate.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
		}

		return messageDate.toLocaleDateString([], {
			month: 'numeric',
			day: 'numeric',
			year: '2-digit',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
	let prevUserId = '';
	function isSameUserIdAsPreviousMessage(userId: string) {
		let isSame = prevUserId === userId;
		prevUserId = userId;
		return isSame;
	}
</script>

<div class="flex h-full w-full">
	<div class="flex h-full flex-1 flex-col overflow-y-auto">
		<div class="mt-auto p-4">
			{#each messages as message}
				{#if message.isSystem}
					<div class="text-center">
						<div class="inline-block rounded-md bg-primary px-3 py-1 text-primary-foreground">
							<p class="font-mono text-xs">{message.content}</p>
						</div>
					</div>
				{:else if !isSameUserIdAsPreviousMessage(message.user_id)}
					<div class="flex items-start pt-3">
						<Avatar class="mr-2 mt-1 h-8 w-8">
							<!-- <AvatarImage src={message.a} alt={message.author.name} /> -->
							<AvatarFallback class="bg-foreground text-xs text-background">
								{message.user_id.substring(0, 2).toUpperCase()}
							</AvatarFallback>
						</Avatar>
						<div class="flex-1">
							<div class="flex items-baseline">
								<span class="font-mono font-bold text-foreground/60">{message.user_id}</span>
								<span class="ml-2 font-mono text-xs text-foreground/40"
									>{timeAgo(message.timestamp)}</span
								>
							</div>
							<p class="font-mono text-sm">{message.contents}</p>
						</div>
					</div>
				{:else}
					<div class="pl-10">
						<p class="font-mono text-sm">{message.contents}</p>
					</div>
				{/if}
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
							class=" border-transparent bg-transparent font-mono focus-visible:outline-0 focus-visible:ring-0 focus-visible:ring-offset-0"
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
</div>
