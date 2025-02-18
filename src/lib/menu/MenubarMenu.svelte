<script lang="ts">
	import { Menubar } from 'bits-ui';

	export let label: string;
</script>

<Menubar.Menu>
	<Menubar.Trigger
		class="mx-[-1px] h-full cursor-default rounded-sm px-3 text-sm text-slate-400 hover:bg-slate-700 hover:text-slate-200"
	>
		{label}
	</Menubar.Trigger>
	<Menubar.Content
		class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-slate-600 bg-slate-800 py-1 shadow-xl"
	>
		<slot />
	</Menubar.Content>
</Menubar.Menu>
