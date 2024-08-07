<script lang="ts">
	import SearchBar from '$lib/components/SearchBar.svelte';
	import { games, setActiveGame } from '$lib/stores';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import { open as openLink } from '@tauri-apps/plugin-shell';
	import { invokeCommand } from '$lib/invoke';
	import Link from './Link.svelte';

	export let onSelect: () => void;

	let shownGames = games;
	let searchTerm = '';

	$: refresh(searchTerm);

	function refresh(searchTerm: string) {
		let lowerSearch = searchTerm.toLowerCase();

		let newGames =
			searchTerm.length > 0
				? games.filter((game) => {
						return (
							game.displayName.toLowerCase().includes(lowerSearch) ||
							game.aliases.some((alias) => alias.toLowerCase().includes(lowerSearch))
						);
					})
				: games;

		newGames.sort((a, b) => {
			if (a.favorite && !b.favorite) return -1;
			if (!a.favorite && b.favorite) return 1;
			return 0;
		});

		shownGames = newGames;
	}
</script>

<div class="relative flex-grow mt-1">
	<SearchBar bind:value={searchTerm} placeholder="Search for games..." />
</div>

<div class="flex flex-col mt-2 h-96 overflow-y-auto">
	{#if shownGames.length > 0}
		{#each shownGames as game}
			<Button.Root
				class="flex hover:bg-gray-700 rounded-lg p-1 items-center group mr-2"
				on:click={() => {
					setActiveGame(game);
					onSelect();
				}}
			>
				<img
					src="games/{game.id}.webp"
					alt={game.displayName}
					class="w-8 h-8 rounded group-hover:shadow-xl mr-2"
				/>

				<span class="flex-grow text-left text-slate-200">
					{game.displayName}
				</span>

				<Button.Root
					class="{game.favorite
						? 'block'
						: 'hidden group-hover:block'} p-1 mr-1 rounded-md hover:bg-gray-600"
					on:click={(evt) => {
						evt.stopPropagation();
						game.favorite = !game.favorite;
						refresh(searchTerm);
						invokeCommand('favorite_game', { id: game.id });
					}}
				>
					<Icon
						icon={game.favorite ? 'mdi:star' : 'mdi:star-outline'}
						class="text-yellow-400 text-xl"
					/>
				</Button.Root>
			</Button.Root>
		{/each}
	{:else}
		<div class="text-slate-300 text-center mt-4">No games found</div>
		<div class="text-slate-400 text-sm max-w-[35rem]">
			Your game missing? If the game is new there's a chance Thunderstore have yet to add it. If you
			can find it on
			<Link href="https://thunderstore.io">thunderstore.io</Link>
			but not here, please message us on
			<Link href="https://discord.com/channels/1168655651455639582/1246088342458863618"
				>Discord</Link
			>
			or open an issue on
			<Link href="https://github.com/Kesomannen/ModManager/issues/">our Github</Link>
		</div>
	{/if}
</div>
