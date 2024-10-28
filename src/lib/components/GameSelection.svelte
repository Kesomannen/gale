<script lang="ts">
	import SearchBar from '$lib/components/SearchBar.svelte';
	import { games, setActiveGame } from '$lib/stores';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
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
						return game.displayName.toLowerCase().includes(lowerSearch);
					})
				: games;

		newGames.sort((a, b) => {
			if (searchTerm.length === 0) {
				if (a.favorite && !b.favorite) return -1;
				if (!a.favorite && b.favorite) return 1;

				if (a.popular && !b.popular) return -1;
				if (!a.popular && b.popular) return 1;
			}

			return a.displayName.localeCompare(b.displayName);
		});

		shownGames = newGames;
	}
</script>

<div class="relative mt-1 flex-grow">
	<SearchBar bind:value={searchTerm} placeholder="Search for games..." />
</div>

<div class="mt-2 flex h-80 flex-col overflow-y-auto">
	{#if shownGames.length > 0}
		{#each shownGames as game}
			<Button.Root
				class="group mr-2 flex items-center rounded-lg p-1 hover:bg-gray-700"
				on:click={() => {
					setActiveGame(game);
					onSelect();
				}}
			>
				<img
					src="games/{game.id}.webp"
					alt={game.displayName}
					class="mr-2 h-8 w-8 rounded group-hover:shadow-xl"
				/>

				<span class="flex-grow text-left text-gray-200">
					{game.displayName}
				</span>

				<Button.Root
					class="{game.favorite
						? 'block'
						: 'hidden group-hover:block'} mr-1 rounded-md p-1 hover:bg-gray-600"
					on:click={(evt) => {
						evt.stopPropagation();
						game.favorite = !game.favorite;
						refresh(searchTerm);
						invokeCommand('favorite_game', { id: game.id });
					}}
				>
					<Icon
						icon={game.favorite ? 'mdi:star' : 'mdi:star-outline'}
						class="text-accent-500 text-xl"
					/>
				</Button.Root>
			</Button.Root>
		{/each}
	{:else}
		<div class="mt-4 text-center text-gray-300">No games found 😢</div>
		<div class="max-w-[35rem] text-sm text-gray-400">
			Your game missing? If the game is new on Thunderstore there's a chance we have yet to add it.
			If you can find it on
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
