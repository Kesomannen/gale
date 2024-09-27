<script lang="ts">
	import InputField from '$lib/components/InputField.svelte';
	import { invoke } from '$lib/invoke';
	import type { Version } from '$lib/models';
	import { games, profiles } from '$lib/state/profile.svelte';
	import { modIconUrl, queueInstall, queueThunderstoreInstall, shortenNum } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { page } from '$app/stores';

	type Package = {
		id: string;
		name: string;
		owner: string;
		description: string;
		isPinned: boolean;
		isDeprecated: boolean;
		ratingScore: number;
		downloads: number;
		hasNsfwContent: boolean;
		dateUpdated: Date;
		versionUuid: string;
	} & Version;

	let searchTerm = $state($page.url.searchParams.get('query') ?? '');

	let packages: Package[] = $state([]);

	let version = 0;

	$effect(() => {
		$page.url.searchParams.set('query', searchTerm);
		queryPackages(searchTerm, ++version);
	});

	async function queryPackages(searchTerm: string, thisVersion: number) {
		let results = await invoke<Package[]>('thunderstore', 'query_packages', {
			args: {
				searchTerm,
				maxResults: 10,
				gameId: games.active?.id,
				orderBy: 'relevance',
				ascending: false
			}
		});

		if (thisVersion === version) {
			packages = results;
		} else {
			console.warn('discarding outdated results');
		}
	}
</script>

<div class="mx-auto w-full max-w-4xl overflow-y-auto px-4">
	<div class="flex items-center pb-2 pt-4">
		<InputField bind:value={searchTerm} placeholder="Search Thunderstore" class="flex-grow" />
	</div>
	<div class="flex flex-col gap-2 overflow-y-auto">
		{#each packages as pkg}
			<div class="rounded-lg border border-gray-500 bg-gray-700 p-4 text-left">
				<div class="flex gap-3">
					<img src={modIconUrl(pkg.owner, pkg.name, pkg)} alt={pkg.name} class="size-16 rounded" />
					<div>
						<span class="text-xl font-bold text-white">{pkg.name}</span>
						<span class="ml-1 text-slate-300">by {pkg.owner}</span>
						<br />
						<span class="text-slate-300">{pkg.description}</span>
					</div>
				</div>
				<div class="mt-2 flex items-center gap-2">
					<div class="flex items-center gap-2 text-yellow-400">
						<Icon icon="akar-icons:star" class="inline" />
						<span>{shortenNum(pkg.ratingScore)}</span>
					</div>
					<div class="ml-4 flex items-center gap-2 text-green-400">
						<Icon icon="akar-icons:download" class="inline" />
						<span>{shortenNum(pkg.downloads)}</span>
					</div>
					<a
						href="/test/{pkg.id}"
						class="ml-auto inline-flex items-center gap-2 rounded-lg bg-gray-600 px-4 py-1.5 font-semibold text-gray-300 transition-all hover:-translate-y-0.5 hover:bg-gray-500 hover:shadow-sm"
					>
						<Icon icon="akar-icons:info" />
						More Info
					</a>
					<div class="inline-flex gap-0.5">
						<button
							class="inline-flex items-center gap-2 rounded-l-lg bg-green-700 px-4 py-1.5 font-semibold text-white transition-all hover:-translate-y-0.5 hover:bg-green-600 hover:shadow-sm"
							onclick={() => queueThunderstoreInstall(pkg.owner, pkg.name, pkg, pkg.versionUuid)}
						>
							<Icon icon="akar-icons:download" />
							Install
						</button>
						<button
							class="inline-flex w-6 items-center rounded-r-lg bg-green-700 font-semibold text-white transition-all hover:-translate-y-0.5 hover:bg-green-600 hover:shadow-sm"
						>
							<Icon icon="material-symbols:arrow-drop-down" class="mx-auto" />
						</button>
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>
