<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import { games, refreshGames, setActiveGame } from '$lib/profile';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import { open as openLink } from '@tauri-apps/api/shell';
	import { invokeCommand } from '$lib/invoke';

	export let open = false;

	let shownGames = games;
	let searchTerm = '';

	$: {
		searchTerm;
		open;
		refresh();
	}

	function refresh() {
		let newGames =
			searchTerm.length > 0
				? games.filter((game) => game.displayName.toLowerCase().includes(searchTerm.toLowerCase()))
				: games;

		newGames.sort((a, b) => {
			if (a.favorite && !b.favorite) return -1;
			if (!a.favorite && b.favorite) return 1;
			return 0;
		});

		shownGames = newGames;
	}
</script>

<Popup title="Browse games" bind:open>
	<div class="relative flex-grow mt-2">
		<input
			type="text"
			class="w-full py-2 pr-10 pl-12 rounded-lg bg-gray-900 text-slate-300 truncate"
			bind:value={searchTerm}
			placeholder="Search for games to mod..."
		/>
		<Icon class="absolute left-[12px] top-[9px] text-slate-400 text-2xl" icon="mdi:magnify" />
	</div>

	<div class="flex flex-col mt-2 h-96 overflow-y-auto">
		{#if shownGames.length > 0}
			{#each shownGames as game}
				<Button.Root
					class="flex hover:bg-gray-700 rounded-lg p-1 items-center group mr-2"
					on:click={() => {
						setActiveGame(game);
						open = false;
					}}
				>
					<img
						src="games/{game.id}.png"
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
							refresh();
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
			<div class="text-slate-400 text-sm">
				Your game missing? If the game is new there's a chance Thunderstore have yet to add it. If
				you can find it on
				<button
					class="hover:underline hover:text-green-500 text-green-600"
					on:click={() => openLink('https://thunderstore.io')}
					>thunderstore.io
				</button>
				but not here, please message us on
				<button
					class="hover:underline hover:text-green-500 text-green-600"
					on:click={() => openLink('https://discord.gg/lcmod')}
					>Discord
				</button>
				or open an issue on
				<button
					class="hover:underline hover:text-green-500 text-green-600"
					on:click={() => openLink('https://github.com/Kesomannen/ModManager/issues/')}
					>our Github
				</button>.
			</div>
		{/if}
	</div>
</Popup>
