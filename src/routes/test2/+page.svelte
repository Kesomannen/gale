<script lang="ts">
	import Checkbox from '$lib/components/Checkbox.svelte';
	import { invoke } from '$lib/invoke';
	import { profiles } from '$lib/state/profile.svelte';
	import Icon from '@iconify/svelte';
	import { Switch } from 'bits-ui';

	let shownMods = $derived.by(() => {
		if (profiles.active === undefined) return [];

		return profiles.active.mods;
	});
</script>

<div class="w-full max-w-4xl mx-auto overflow-y-auto px-4 pt-4">
	<table class="w-full">
		<thead>
			<tr class="text-gray-200 bg-gray-900">
				<th></th>
				<th class="text-left py-2">Name</th>
				<th class="text-left">Author</th>
				<th class="text-left">Version</th>
				<th class="text-left">Actions</th>
			</tr>
		</thead>
		<tbody>
			{#each shownMods as { id, owner, name, icon, version, enabled, href }, i}
				<tr class="even:bg-gray-900 odd:bg-gray-800">
					<td class="flex pl-4 pr-2 my-auto">
						<Checkbox value={false} />
					</td>
					<td class="py-2">
						<img src={icon ?? ''} alt={name.slice(0, 2)} class="size-10 inline-block rounded" />
						<a
							class="font-medium hover:underline hover:text-green-400 truncate pl-1"
							href={href ?? ''}
							class:line-through={!enabled}
              class:text-gray-300={!enabled}
              class:text-white={enabled}
              >{name}</a
						></td
					>
					<td class="text-gray-300 truncate"
						>{#if owner !== null}
							{owner}
						{/if}</td
					>
					<td class="text-gray-300">{version ?? ''}</td>
					<td class="flex items-center py-3">
						<Switch.Root
							checked={enabled}
							onCheckedChange={() => invoke('profile', 'force_toggle_mod', { id })}
							on:click={(evt) => evt.stopPropagation()}
							class="flex px-1 py-1 mr-1 rounded-full w-12 h-6 group bg-gray-700 hover:bg-gray-600 data-[state=checked]:bg-green-700 data-[state=checked]:hover:bg-green-600"
						>
							<Switch.Thumb
								class="pointer-events-none h-full w-4 rounded-full transition-transform ease-out duration-75 bg-gray-400 hover:bg-gray-300 data-[state=checked]:translate-x-6 data-[state=checked]:bg-green-300 data-[state=checked]:group-hover:bg-green-200"
							/>
						</Switch.Root>
						<button
							class="text-gray-400 hover:bg-red-700 hover:text-white p-1.5 rounded-lg text-lg"
							onclick={() => invoke('profile', 'force_uninstall_mod', { id })}
						>
							<Icon icon="material-symbols:delete" />
						</button>
						<button
							class="text-gray-400 hover:bg-gray-700 hover:text-white p-1.5 rounded-full text-lg"
						>
							<Icon icon="material-symbols:more-vert" />
						</button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
