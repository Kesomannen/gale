<script lang="ts">
	import type { ConfigFile, ConfigSection } from "$lib/models";
	import Icon from "@iconify/svelte";
	import { Button, Collapsible } from "bits-ui";
	import { quadOut } from "svelte/easing";
	import { slide } from "svelte/transition";

    export let file: ConfigFile;
    export let selectedSection: ConfigSection | undefined;
    export let onClick: (file: ConfigFile, section: ConfigSection) => void;

    let open = false;
</script>

<Collapsible.Root bind:open={open}>
    <Collapsible.Trigger class="flex pl-3 pr-2 w-full text-slate-200 hover:text-white items-center group"> 
        <Icon
            icon="mdi:expand-more"
            class="text-xl {open ? 'rotate-180' : 'rotate-0'} transition-all mr-1 flex-shrink-0"
        />
        <div class="truncate flex-shrink">
            {file.name}
        </div>
        <Button.Root 
            class="ml-auto flex-shrink-0 hidden group-hover:inline text-slate-400 p-1 rounded hover:text-slate-200 hover:bg-gray-600"
        >
            <Icon icon="mdi:delete" />
        </Button.Root>
        <Button.Root 
            class="flex-shrink-0 hidden group-hover:inline text-slate-400 p-1 rounded hover:text-slate-200 hover:bg-gray-600"
        >
            <Icon icon="mdi:open-in-new" />
        </Button.Root>
    </Collapsible.Trigger>
    <Collapsible.Content
        class="flex flex-col mt-0.5 mb-1" 
        transition={slide}
        transitionConfig={{ duration: 100, easing: quadOut }}
    >
        {#each file.sections as section}
            <Button.Root 
                class="text-sm pl-9 pr-2 py-0.5 text-left truncate
                      {selectedSection === section ? 'bg-gray-600 text-slate-200 font-medium' : 'text-slate-300 hover:bg-gray-600 hover:text-slate-200 font-light'}"
                on:click={() => onClick(file, section)}
            >
                {section.name}
            </Button.Root>
        {/each}
    </Collapsible.Content>
</Collapsible.Root>