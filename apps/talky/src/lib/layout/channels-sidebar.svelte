<script lang="ts">
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { page } from '$app/state';
	import { withChannelList } from '$lib/channel-list.svelte';
	import { Button } from '$lib/components/ui/button';
	import { cn } from '$lib/utils';
	import { AudioLines, ChevronUp, Hash, MessageSquare } from 'lucide-svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { user } from '$lib/user.svelte';

	const channels = withChannelList();
</script>

<Sidebar.Root>
	<Sidebar.Content>
		{#if user.user}
			<Sidebar.Group>
				<Sidebar.GroupLabel>Application</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each channels.channels as channel}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton>
									{#snippet child({ props })}
										<a
											href="/of/devils/{channel.slug}"
											{...props}
											class={cn(
												props.class || '',
												page.params.channel === channel.slug ? 'bg-sidebar-accent/55' : ''
											)}
										>
											<div class="flex flex-1 items-center overflow-hidden">
												{#if channel.type === 'chat'}
													<MessageSquare class="mr-1.5 h-4 w-4 flex-shrink-0" />
												{:else if channel.type === 'feed'}
													<Hash class="mr-1.5 h-4 w-4 flex-shrink-0" />
												{:else}
													<AudioLines class="mr-1.5 h-4 w-4 flex-shrink-0" />
												{/if}
												<span class="truncate font-light">{channel.name}</span>
											</div>
										</a>
									{/snippet}
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/if}
	</Sidebar.Content>
	<Sidebar.Footer>
		{#if user.user}
			<Sidebar.Menu>
				<Sidebar.MenuItem>
					<DropdownMenu.Root>
						<DropdownMenu.Trigger>
							{#snippet child({ props })}
								<Sidebar.MenuButton
									{...props}
									class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
								>
									{user.user!.sub}
									<ChevronUp class="ml-auto" />
								</Sidebar.MenuButton>
							{/snippet}
						</DropdownMenu.Trigger>
						<DropdownMenu.Content side="top" class="w-[--bits-dropdown-menu-anchor-width]">
							<DropdownMenu.Item>
								<span>Sign out</span>
							</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				</Sidebar.MenuItem>
			</Sidebar.Menu>
		{:else}
			<div class="flex gap-1.5">
				<Button class="w-1/2" variant="secondary" href="/login">Log in</Button>
				<Button class="w-1/2" variant="link" href="/register">Sign up</Button>
			</div>
		{/if}
	</Sidebar.Footer></Sidebar.Root
>
