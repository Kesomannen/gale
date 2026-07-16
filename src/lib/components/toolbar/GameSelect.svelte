<script lang="ts">
	import GameSelectItem from './GameSelectItem.svelte';

	import SearchBar from '$lib/components/ui/SearchBar.svelte';
	import Link from '$lib/components/ui/Link.svelte';
	import games from '$lib/state/game.svelte';
	import { m } from '$lib/paraglide/messages';
	import HelpCard from '../ui/HelpCard.svelte';

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
			<HelpCard title={m.gameSelect_title()} icon="mdi:magnify">
				<div class="text-primary-400 mt-2 max-w-140 text-sm">
					{m.gameSelect_content_1()}
					<Link href="https://github.com/Kesomannen/ModManager/issues/new"
						>{m.gameSelect_content_2()}</Link
					>
					{m.gameSelect_content_3()}
				</div>
			</HelpCard>
		{/if}
	</div>
</div>
