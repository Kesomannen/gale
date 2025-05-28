<script lang="ts">
	import SearchBar from '$lib/components/SearchBar.svelte';
	import { activeGame, games, setActiveGame } from '$lib/stores';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import { invokeCommand } from '$lib/invoke';
	import Link from '../components/Link.svelte';
	import { titleCase } from '$lib/util';

	export let onSelect: () => void;

	let shownGames = games;
	let searchTerm = '';

	$: refresh(searchTerm);

	function refresh(searchTerm: string) {
		let lowerSearch = searchTerm.toLowerCase();

		let newGames =
			searchTerm.length > 0
				? games.filter((game) => {
						return game.name.toLowerCase().includes(lowerSearch);
					})
				: games;

		newGames.sort((a, b) => {
			if (searchTerm.length === 0) {
				if (a.favorite && !b.favorite) return -1;
				if (!a.favorite && b.favorite) return 1;

				if (a.popular && !b.popular) return -1;
				if (!a.popular && b.popular) return 1;
			}

			return a.name.localeCompare(b.name);
		});

		shownGames = newGames;
	}
</script>

<div class="mt-1">
	<div class="relative grow">
		<SearchBar bind:value={searchTerm} placeholder="Search for games..." />
	</div>

	<div class="mt-1 flex h-80 flex-col overflow-y-scroll">
		{#if shownGames.length > 0}
			{#each shownGames as game}
				<Button.Root
					class="group hover:bg-primary-700 mr-2 flex items-center rounded-lg border p-1.5 {$activeGame?.slug ===
					game.slug
						? ' border-primary-500 bg-primary-700'
						: 'hover:bg-primary-700 border-transparent'}"
					on:click={() => {
						setActiveGame(game.slug);
						onSelect();
					}}
				>
					<img src="games/{game.slug}.webp" alt={game.name} class="mr-2 size-12 rounded-sm" />

					<div class="grow pl-1 text-left">
						<div class="font-medium text-white">
							{game.name}
						</div>

						<div class="text-primary-400">
							<span>{game.modLoader} </span>

							<!--
							{#if game.modLoader !== ModLoader.BepInEx}
								<Tooltip
									class="inline-flex rounded-sm bg-red-600 p-0.5 text-sm text-white"
									text="Experimental support. Here be dragons!"
								>
									<Icon icon="mdi:beta" />
								</Tooltip>
							{/if}
							-->

							{#if game.platforms.length > 0}
								<span class="text-primary-500 mx-1">|</span>

								<span class="mr-1">{game.platforms.map(titleCase).join(', ')}</span>
							{/if}
						</div>
					</div>

					<Button.Root
						class="hover:bg-primary-600 mr-1 rounded p-1.5 {game.favorite
							? 'block'
							: 'hidden group-hover:block'}"
						on:click={(evt) => {
							evt.stopPropagation();
							game.favorite = !game.favorite;
							refresh(searchTerm);
							invokeCommand('favorite_game', { slug: game.slug });
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
			<div class="text-primary-300 mt-4 text-center">No games found</div>
			<div class="text-primary-400 mt-2 max-w-[35rem] text-sm">
				Your game missing? If the game is new on Thunderstore there's a chance we have yet to add
				it. If you can find it on
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
</div>
