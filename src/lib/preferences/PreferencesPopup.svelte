<script lang="ts">
	import { Tabs } from 'bits-ui';
	import Popup from '../Popup.svelte';
	import PathSetting from './PathSetting.svelte';

	export let open = false;
</script>

<Popup title="Preferences" bind:open>
	<Tabs.Root value="paths">
		<Tabs.List class="flex gap-3 justify-stretch my-3">
			<Tabs.Trigger
				value="paths"
				class="px-3 py-0.5 text-slate-400 border-white
					data-[state=active]:text-white data-[state=active]:border-b data-[state=active]:font-medium"
			>
				Paths
			</Tabs.Trigger>
			<Tabs.Trigger 
				value="other"
				class="px-3 py-0.5 text-slate-400 border-white
					data-[state=active]:text-white data-[state=active]:border-b data-[state=active]:font-medium"
			>
				Other
			</Tabs.Trigger>
		</Tabs.List>
		<Tabs.Content value="paths" class="flex flex-col gap-1">
			<PathSetting label="Steam executable" name="steam_exe_path" type="exe" />
			<PathSetting label="Download cache directory" name="cache_path" type="dir" />
			<PathSetting label="Data directory" name="data_path" type="dir" />
		</Tabs.Content>
		<Tabs.Content value="other" class="flex flex-col gap-1">
			Hello
		</Tabs.Content>
	</Tabs.Root>
</Popup>
