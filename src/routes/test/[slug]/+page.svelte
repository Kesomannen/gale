<script lang="ts">
	import { page } from '$app/stores';
	import Markdown from '$lib/components/Markdown.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';
	import {
		modIconUrl,
		shortenFileSize,
		shortenNum,
		timeSince,
		modThunderstoreUrl,
		queueInstall,
		queueThunderstoreInstall
	} from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Tabs } from 'bits-ui';
	import { invoke } from '$lib/invoke';
	import type { Version } from '$lib/models';
	import Button from '$lib/components/Button.svelte';

	let id = $page.params.slug;

	let promise = invoke<{
		name: string;
		owner: string;
		description: string;
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

<div class="mx-auto flex w-full max-w-6xl gap-4 overflow-hidden px-4 pt-4">
	{#await promise}
		<div class="flex h-full w-full items-center justify-center text-xl text-gray-400">
			<Icon icon="mdi:loading" class="mr-4 animate-spin" />
			Loading...
		</div>
	{:then { name, owner, downloads, ratingScore, versions, description, readme, changelog, websiteUrl, donationUrl, dependencies }}
		<div class="w-[25%] min-w-56 flex-shrink-0 overflow-y-hidden">
			<img src={modIconUrl(owner, name, versions[0])} alt={name} class="w-full rounded-lg" />
			<h1 class="break-words pt-3 text-3xl font-bold text-white">
				{name.replace(/_/g, ' ')}
			</h1>
			<h3 class="truncate text-xl text-gray-200">by {owner}</h3>

			<p class="pb-3 text-gray-400">
				{description}
			</p>

			<div class="flex flex-wrap gap-2 pb-1">
				<Button
					color="green"
					class="flex-grow"
					label="Install"
					icon="material-symbols:download"
					onclick={() => queueThunderstoreInstall(owner, name, versions[0], versions[0].id)}
				/>
				<Button
					color="gray"
					secondary
					class="flex-grow"
					label="Thunderstore"
					icon="material-symbols:open-in-new"
					href={modThunderstoreUrl(owner, name)}
				/>
			</div>

			<div class="space-y-1 py-3">
				{#if donationUrl}
					{@render link('Donate', 'mdi:heart', donationUrl)}
				{/if}

				{#if websiteUrl}
					{@render link('Website', 'material-symbols:link', websiteUrl)}
				{/if}

				{@render link(
					'Thunderstore',
					'material-symbols:open-in-new',
					modThunderstoreUrl(owner, name)
				)}
			</div>

			<div class="space-y-1 py-2 text-gray-300">
				<div>
					<Icon icon="material-symbols:star" class="mr-1 inline" />
					{shortenNum(ratingScore)}
				</div>
				<div>
					<Icon icon="material-symbols:download" class="mr-1 inline" />
					{shortenNum(downloads)}
				</div>
				<div>
					<Icon icon="material-symbols:calendar-clock" class="mr-1 inline" />
					{timeSince(new Date(versions[0].dateCreated))} ago
				</div>
				<div>
					<Icon icon="material-symbols:weight" class="mr-1 inline" />
					{shortenFileSize(versions[0].fileSize)}
				</div>
			</div>
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
				<Tabs.Content
					value="readme"
					class="h-full overflow-y-auto rounded-xl bg-gray-800 outline-none"
				>
					<Markdown source={readme} />
				</Tabs.Content>
				<Tabs.Content value="changelog" class="h-full overflow-y-auto outline-none">
					<Markdown source={changelog ?? 'No changelog available'} />
				</Tabs.Content>
				<Tabs.Content value="dependencies" class="grid grid-cols-2 overflow-y-auto">
					{#each dependencies.sort((a, b) => a.name.localeCompare(b.name)) as dependency}
						<div class="flex items-center gap-2 p-2">
							<img
								src={modIconUrl(dependency.owner, dependency.name, dependency)}
								alt={dependency.name}
								class="size-12 rounded"
							/>

							<div>
								<a
									class="text-lg font-semibold text-white hover:underline"
									href="/test/{dependency.id}/">{dependency.name}</a
								>
								<div class="text-gray-300">by {dependency.owner}</div>
							</div>
						</div>
					{/each}
				</Tabs.Content>
				<Tabs.Content value="versions" class="h-full overflow-y-auto">
					<table class="mt-2 w-full rounded border-2 border-gray-900">
						<tbody>
							{#each versions as version}
								<tr class="odd:bg-gray-900">
									<td class="px-2 py-1.5 text-lg font-semibold text-white">
										{version.major}.{version.minor}.{version.patch}
									</td>
									<td class="text-gray-400">
										<Icon icon="material-symbols:calendar-clock" class="mr-1 inline" />
										{timeSince(new Date(version.dateCreated))} ago
									</td>
									<td class="text-gray-400">
										<Icon icon="material-symbols:download" class="mr-1 inline" />
										{shortenNum(version.downloads)}
									</td>
									<td>
										<Button
											tertiary
											color="green"
											label="Install"
											icon="material-symbols:download"
											onclick={() => queueThunderstoreInstall(owner, name, version, version.id)}
										/>
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
		<Icon {icon} class="mr-1 inline text-gray-300" />
		<a {href} target="_blank" class="text-green-400 hover:text-green-300 hover:underline">
			{text}
		</a>
	</div>
{/snippet}
