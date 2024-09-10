<script lang="ts">
	import { page } from '$app/stores';
	import Markdown from '$lib/components/Markdown.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';
	import { invokeCommand } from '$lib/invoke';
	import { shortenFileSize, shortenNum } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Tabs } from 'bits-ui';

	let id = $page.params.slug;

	let promise = invokeCommand<{
		name: string;
		owner: string;
		readme: string;
		changelog: string | null;
		websiteUrl: string | null;
		donationUrl: string | null;
		versions: {
			major: number;
			minor: number;
			patch: number;
			fileSize: number;
			downloads: number;
		}[];
	}>('plugin:gale-thunderstore|query_package', { id });

	let tab: 'readme' | 'changelog' | 'versions' = 'readme';
</script>

<div class="flex gap-6 w-full max-w-screen-xl mx-auto overflow-hidden px-4 pt-4">
	{#await promise}
		<p>loading...</p>
	{:then { name, owner, versions, readme, changelog, websiteUrl, donationUrl }}
		<div class="flex-shrink-0 w-1/4 min-w-64 overflow-hidden">
			<img
				src="https://gcdn.thunderstore.io/live/repository/icons/{owner}-{name}-{versions[0]
					.major}.{versions[0].minor}.{versions[0].patch}.png"
				alt={name}
				class="w-full rounded-lg"
			/>
			<h1 class="text-white font-bold text-3xl pt-6 truncate">{name}</h1>
			<h3 class="text-slate-300 text-xl truncate">by {owner}</h3>
			<button
				class="flex items-center justify-center gap-2 bg-green-700 text-white mt-4 py-2 w-full rounded-lg
				hover:bg-green-600 hover:-translate-y-0.5 transition-all hover:shadow-sm"
			>
				<Icon icon="akar-icons:download" />
				Install latest
			</button>
		</div>
		<div class="flex-grow overflow-hidden">
			<TabsMenu
				options={[
					{ value: 'readme', label: 'Details' },
					{ value: 'changelog', label: 'Changelog' },
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
				<Tabs.Content value="versions" class="overflow-y-auto h-full">
					<table class="w-full">
						<tbody>
							{#each versions as version}
								<tr class="odd:bg-gray-900">
									<td class="text-white font-semibold text-lg py-1 px-2">
										{version.major}.{version.minor}.{version.patch}
									</td>
									<td class="text-slate-400">
										<Icon icon="material-symbols:weight" class="inline mr-1" />
										{shortenFileSize(version.fileSize)}
									</td>
									<td class="text-slate-400">
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
