<script lang="ts">
	import GameSelectItem from './GameSelectItem.svelte';

	import SearchBar from '$lib/components/ui/SearchBar.svelte';
	import Link from '$lib/components/ui/Link.svelte';
	import games from '$lib/state/game.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		onselect: () => void;
	};

	let { onselect }: Props = $props();

	let shownGames = $state(games.list);
	let searchTerm = $state('');

	function refresh(searchTerm: string) {
		let lowerSearch = searchTerm.toLowerCase();

		let newGames =
			searchTerm.length > 0
				? games.list.filter((game) => {
						return game.name.toLowerCase().includes(lowerSearch);
					})
				: games.list;

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

	$effect(() => {
		refresh(searchTerm);
	});
</script>

<div class="mt-1">
	<div class="relative grow">
		<SearchBar bind:value={searchTerm} placeholder={m.gameSelect_placeholder()} />
	</div>

	<div class="mt-1 flex h-80 flex-col overflow-y-scroll">
		{#if shownGames.length > 0}
			{#each shownGames as game}
				<GameSelectItem
					{game}
					{onselect}
					onfavorite={(favorite) => {
						game.favorite = favorite;
						refresh(searchTerm);
					}}
				/>
			{/each}
		{:else}
			<div class="text-primary-300 mt-4 text-center">{m.gameSelect_content_1()}</div>
			<div class="text-primary-400 mt-2 max-w-[35rem] text-sm">
				{m.gameSelect_content_2()}
				<Link href="https://thunderstore.io">thunderstore.io</Link>
				{m.gameSelect_content_3()}
				<Link href="https://discord.com/channels/1168655651455639582/1246088342458863618"
					>Discord</Link
				>
				{m.gameSelect_content_4()}
				<Link href="https://github.com/Kesomannen/ModManager/issues/"
					>{m.gameSelect_content_5()}</Link
				>
			</div>
		{/if}
	</div>
</div>
