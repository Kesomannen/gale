<script lang="ts">
	import { page } from '$app/stores';
	import Markdown from '$lib/components/Markdown.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';
	import { modIconUrl, shortenFileSize, shortenNum, timeSince, modThunderstoreUrl, queueInstall, queueThunderstoreInstall } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Tabs } from 'bits-ui';
	import { invoke } from '$lib/invoke';
	import type { Version } from '$lib/models';

	let id = $page.params.slug;
	
	let promise = invoke<{
		name: string;
		owner: string;
		readme: string;
		changelog: string | null;
		websiteUrl: string | null;
		donationUrl: string | null;
		downloads: number;
		ratingScore: number;
		versions: (Version & {
			id: string;
			fileSize: number;
			downloads: number;
			dateCreated: string;
		})[];
		dependencies: (Version & { id: String; name: string; owner: string })[];
	}>('thunderstore', 'query_package', { id });

	enum Tab {
		README = 'readme',
		CHANGELOG = 'changelog',
		DEPENDENCIES = 'dependencies',
		VERSIONS = 'versions'
	}

	let tab: Tab = $state(Tab.README);
</script>

<div class="flex gap-6 w-full max-w-6xl mx-auto overflow-hidden px-4 pt-4">
	{#await promise}
		<div class="text-gray-400 w-full h-full flex items-center justify-center text-xl">
			<Icon icon="mdi:loading" class="animate-spin mr-4" />
			Loading...
		</div>
	{:then { name, owner, downloads, ratingScore, versions, readme, changelog, websiteUrl, donationUrl, dependencies }}
		<div class="flex-shrink-0 w-1/4 min-w-56 overflow-hidden">
			<img src={modIconUrl(owner, name, versions[0])} alt={name} class="w-full rounded-lg" />
			<h1 class="text-white font-bold text-3xl pt-3 truncate">{name}</h1>
			<h3 class="text-gray-300 text-xl truncate">by {owner}</h3>

			<div class="pt-4 space-y-1">
				<div class="text-gray-300">
					<Icon icon="material-symbols:star" class="inline mr-1" />
					{shortenNum(ratingScore)}
				</div>
				<div class="text-gray-300">
					<Icon icon="material-symbols:download" class="inline mr-1" />
					{shortenNum(downloads)}
				</div>
				<div class="text-gray-300">
					<Icon icon="material-symbols:calendar-clock" class="inline mr-1" />
					{timeSince(new Date(versions[0].dateCreated))} ago
				</div>
				<div class="text-gray-300">
					<Icon icon="material-symbols:weight" class="inline mr-1" />
					{shortenFileSize(versions[0].fileSize)}
				</div>
			</div>

			<div class="pb-6 pt-4 space-y-1">
				{#if donationUrl}
					{@render link('Donate', 'mdi:heart', donationUrl)}
				{/if}

				{#if websiteUrl}
					{@render link('Website', 'material-symbols:link-rounded', websiteUrl)}
				{/if}

				{@render link('Thunderstore', 'material-symbols:open-in-new', modThunderstoreUrl(owner, name))}
			</div>

			<button
				class="flex items-center justify-center gap-2 bg-green-700 text-white py-2 w-full rounded-lg hover:bg-green-800 hover:-translate-y-0.5 transition-all hover:shadow-sm"
				onclick={() => queueThunderstoreInstall(owner, name, versions[0], versions[0].id)}
			>
				<Icon icon="material-symbols:download" />
				Install latest
			</button>
		</div>
		<div class="flex-grow overflow-hidden">
			<TabsMenu
				options={[
					{ value: Tab.README, label: 'Details', disabled: !readme },
					{ value: Tab.CHANGELOG, label: 'Changelog', disabled: !changelog },
					{
						value: Tab.DEPENDENCIES,
						label: `Dependencies (${dependencies.length})`,
						disabled: dependencies.length === 0
					},
					{ value: Tab.VERSIONS, label: 'Versions' }
				]}
				bind:value={tab as Tab}
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
								src={modIconUrl(dependency.owner, dependency.name, dependency)}
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
					<table class="w-full mt-2 rounded border-2 border-gray-900">
						<tbody>
							{#each versions as version}
								<tr class="odd:bg-gray-900">
									<td class="text-white font-semibold text-lg py-1.5 px-2">
										{version.major}.{version.minor}.{version.patch}
									</td>
									<td class="text-gray-400">
										<Icon icon="material-symbols:calendar-clock" class="inline mr-1" />
										{timeSince(new Date(version.dateCreated))} ago
									</td>
									<td class="text-gray-400">
										<Icon icon="material-symbols:download" class="inline mr-1" />
										{shortenNum(version.downloads)}
									</td>
									<td>
										<button
											class="inline-flex items-center gap-2 text-green-400 font-medium hover:text-green-300 hover:underline"
											onclick={() => queueThunderstoreInstall(owner, name, version, version.id)}
										>
											<Icon icon="material-symbols:download" />
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

{#snippet link(text: string, icon: string, href: string)}
	<div>
		<Icon {icon} class="inline mr-1 text-gray-300" />
		<a
			{href}
			target="_blank"
			class="text-green-400 hover:text-green-300 hover:underline"
		>
			{text}
		</a>
	</div>
{/snippet}
