<script lang="ts">
	import { page } from '$app/stores';
	import Markdown from '$lib/components/Markdown.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';
	import { invokeCommand } from '$lib/invoke';
	import { shortenFileSize, shortenNum, timeSince } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Tabs } from 'bits-ui';

	let id = $page.params.slug;

	type Version = {
		major: number;
		minor: number;
		patch: number;
	};

	let promise = invokeCommand<{
		name: string;
		owner: string;
		readme: string;
		changelog: string | null;
		websiteUrl: string | null;
		donationUrl: string | null;
		downloads: number;
		ratingScore: number;
		versions: (Version & {
			fileSize: number;
			downloads: number;
			dateCreated: string;
		})[];
		dependencies: (Version & { id: String; name: string; owner: string })[];
	}>('plugin:gale-thunderstore|query_package', { id });

	let tab: 'readme' | 'changelog' | 'dependencies' | 'versions' = 'readme';

	function iconUrl(owner: string, name: string, version: Version) {
		return `https://gcdn.thunderstore.io/live/repository/icons/${owner}-${name}-${version.major}.${version.minor}.${version.patch}.png`;
	}
</script>

<div class="flex gap-6 w-full max-w-6xl mx-auto overflow-hidden px-4 pt-4">
	{#await promise}
		<div class="text-gray-400 w-full h-full flex items-center justify-center text-xl">
			<Icon icon="mdi:loading" class="animate-spin mr-4" />
			Loading...
		</div>
	{:then { name, owner, downloads, ratingScore, versions, readme, changelog, websiteUrl, donationUrl, dependencies }}
		<div class="flex-shrink-0 w-1/4 min-w-56 overflow-hidden">
			<img src={iconUrl(owner, name, versions[0])} alt={name} class="w-full rounded-lg" />
			<h1 class="text-white font-bold text-3xl pt-3 truncate">{name}</h1>
			<h3 class="text-gray-300 text-xl truncate">by {owner}</h3>

			<div class="pt-4 space-y-1">
				<div class="text-gray-300">
					<Icon icon="akar-icons:star" class="inline mr-1" />
					{shortenNum(ratingScore)}
				</div>
				<div class="text-gray-300">
					<Icon icon="akar-icons:download" class="inline mr-1" />
					{shortenNum(downloads)}
				</div>
				<div class="text-gray-300">
					<Icon icon="akar-icons:calendar" class="inline mr-1" />
					{timeSince(new Date(versions[0].dateCreated))} ago
				</div>
			</div>

			<div class="pb-6 pt-4 space-y-1">
				{#if websiteUrl}
					<div>
						<Icon icon="akar-icons:link" class="inline mr-1 text-gray-300" />
						<a
							href={websiteUrl}
							target="_blank"
							rel="noopener noreferrer"
							class="text-green-400 hover:text-green-300 hover:underline"
						>
							Website
						</a>
					</div>
				{/if}

				{#if donationUrl}
					<div>
						<Icon icon="akar-icons:heart" class="inline mr-1 text-gray-300" />
						<a
							href={donationUrl}
							target="_blank"
							rel="noopener noreferrer"
							class="text-green-400 hover:text-green-300 hover:underline"
						>
							Donate
						</a>
					</div>
				{/if}

				<div>
					<Icon icon="akar-icons:link-out" class="inline mr-1 text-gray-300" />
					<a
						href={`https://thunderstore.io/package/${owner}/${name}/`}
						target="_blank"
						rel="noopener noreferrer"
						class="text-green-400 hover:text-green-300 hover:underline"
					>
						Thunderstore
					</a>
				</div>
			</div>

			<button
				class="flex items-center justify-center gap-2 bg-green-700 text-white py-2 w-full rounded-lg
				hover:bg-green-800 hover:-translate-y-0.5 transition-all hover:shadow-sm"
			>
				<Icon icon="akar-icons:download" />
				Install latest
			</button>
		</div>
		<div class="flex-grow overflow-hidden">
			<TabsMenu
				options={[
					{ value: 'readme', label: 'Details', disabled: !readme },
					{ value: 'changelog', label: 'Changelog', disabled: !changelog },
					{
						value: 'dependencies',
						label: `${dependencies.length} dependencies`,
						disabled: dependencies.length === 0
					},
					{ value: 'versions', label: 'Versions' }
				]}
				bind:value={tab}
			>
				<Tabs.Content value="readme" class="overflow-y-auto h-full outline-none">
					<Markdown source={readme} />
				</Tabs.Content>
				<Tabs.Content value="changelog" class="overflow-y-auto h-full outline-none">
					<Markdown source={changelog ?? 'No changelog available'} />
				</Tabs.Content>
				<Tabs.Content value="dependencies" class="overflow-y-auto grid grid-cols-2">
					{#each dependencies.sort((a, b) => a.name.localeCompare(b.name)) as dependency}
						<div class="flex items-center gap-2 p-2">
							<img
								src={iconUrl(dependency.owner, dependency.name, dependency)}
								alt={dependency.name}
								class="size-12 rounded"
							/>

							<div>
								<a
									class="text-white font-semibold text-lg hover:underline"
									href="/test/{dependency.id}/">{dependency.name}</a
								>
								<div class="text-gray-300">by {dependency.owner}</div>
							</div>
						</div>
					{/each}
				</Tabs.Content>
				<Tabs.Content value="versions" class="overflow-y-auto h-full">
					<table class="w-full mt-2">
						<tbody>
							{#each versions as version}
								<tr class="odd:bg-gray-900">
									<td class="text-white font-semibold text-lg py-1.5 px-2">
										{version.major}.{version.minor}.{version.patch}
									</td>
									<td class="text-gray-400">
										<Icon icon="material-symbols:weight" class="inline mr-1" />
										{shortenFileSize(version.fileSize)}
									</td>
									<td class="text-gray-400">
										<Icon icon="akar-icons:download" class="inline mr-1" />
										{shortenNum(version.downloads)}
									</td>
									<td>
										<button
											class="inline-flex items-center gap-2 text-green-500 font-semibold
											hover:text-green-400 transition-all hover:underline"
										>
											<Icon icon="akar-icons:download" />
											Install
										</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</Tabs.Content>
			</TabsMenu>
		</div>
	{:catch error}
		<p>{error.message}</p>
	{/await}
</div>
