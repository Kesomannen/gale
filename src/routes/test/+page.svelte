<script lang="ts">
	import InputField from '$lib/components/InputField.svelte';
	import { invokeCommand } from '$lib/invoke';
	import { shortenNum } from '$lib/util';
	import Icon from '@iconify/svelte';

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
		major: number;
		minor: number;
		patch: number;
	};

	let searchTerm = 'bepinex';
	let loading = false;
	let packages: Package[] = [];

	$: queryPackages(searchTerm);

	async function queryPackages(searchTerm: string) {
		if (searchTerm.length === 0) {
			loading = false;
			packages = [];
			return;
		}

		loading = true;
		packages = await invokeCommand<Package[]>('plugin:gale-thunderstore|query_packages', {
			args: {
				searchTerm,
				maxResults: 10,
				communityId: 2
			}
		});

		loading = false;
	}
</script>

<div class="w-full max-w-screen-md mx-auto overflow-y-auto px-4">
	<div class="flex items-center pt-4 pb-2">
		<InputField bind:value={searchTerm} placeholder="Search Thunderstore" class="flex-grow" />
	</div>
	<div class="flex flex-col overflow-y-auto gap-2">
		{#each packages as pkg}
			<div class="text-left p-4 rounded-lg bg-gray-700 border border-gray-500">
				<div class="flex gap-3">
					<img
						src="https://gcdn.thunderstore.io/live/repository/icons/{pkg.owner}-{pkg.name}-{pkg.major}.{pkg.minor}.{pkg.patch}.png"
						alt={pkg.name}
						class="size-16 rounded"
					/>
					<div>
						<span class="text-white font-bold text-xl">{pkg.name}</span>
						<span class="text-slate-300 ml-1">by {pkg.owner}</span>
						<br />
						<span class="text-slate-300">{pkg.description}</span>
					</div>
				</div>
				<div class="flex gap-2 mt-2 items-center">
					<div class="flex items-center gap-2 text-yellow-400">
						<Icon icon="akar-icons:star" class="inline" />
						<span>{shortenNum(pkg.ratingScore)}</span>
					</div>
					<div class="flex items-center gap-2 ml-4 text-green-400">
						<Icon icon="akar-icons:download" class="inline" />
						<span>{shortenNum(pkg.downloads)}</span>
					</div>
					<a
						href="/test/{pkg.id}"
						class="inline-flex items-center gap-2 ml-auto bg-gray-600 text-gray-300 font-semibold py-1.5 px-4 rounded-lg
                    		hover:bg-gray-500 hover:-translate-y-0.5 transition-all hover:shadow-sm"
					>
						<Icon icon="akar-icons:info" />
						More Info
					</a>
					<div class="inline-flex gap-0.5">
						<button
							class="inline-flex items-center gap-2 bg-green-700 text-white font-semibold py-1.5 px-4 rounded-l-lg
                    			hover:bg-green-600 hover:-translate-y-0.5 transition-all hover:shadow-sm"
						>
							<Icon icon="akar-icons:download" />
							Install
						</button>
						<button
							class="inline-flex items-center bg-green-700 text-white font-semibold w-6 rounded-r-lg
                  				hover:bg-green-600 hover:-translate-y-0.5 transition-all hover:shadow-sm"
						>
							<Icon icon="material-symbols:arrow-drop-down" class="mx-auto" />
						</button>
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>
