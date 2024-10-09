/**
 * The following file is generated through a build script. Manually modifying it is an at-your-own-risk activity and your changes will likely be overridden.
 */

(function () {    
    class ExampleExtension extends Extension {
        constructor(ide) {
            super('Example Extension');
        }

        onOpenRole() {

        }

        getSettings() {
            return [
				Extension.ExtensionSetting.createFromLocalStorage('All Caps output from Menu Item', 'exampleextensionallcaps', false, 'Capitalize output', 'Do not capitalize output', false),

            ];
        }

        getMenu() {
            return {
				'Print Hello World': window.ExampleExtension_fns.print_hello_world,
				'Print Extension Name': window.ExampleExtension_fns.print_extension_name,

            };
        }

        getCategories() {
            return [
				new Extension.Category('Hello World', new Color(100, 149, 237)),

            ];
        }

        getPalette() {
            return [
				new Extension.PaletteCategory(
					'Hello World',
					[
						new Extension.Palette.Block('logHelloWorld'),
						new Extension.Palette.Block('logHelloName'),
					],
					SpriteMorph
				),
				new Extension.PaletteCategory(
					'Hello World',
					[
						new Extension.Palette.Block('logHelloWorld'),
						new Extension.Palette.Block('logHelloName'),
					],
					StageMorph
				),
				new Extension.PaletteCategory(
					'control',
					[
						new Extension.Palette.Block('receiveTestEvent'),
						new Extension.Palette.Block('printProcess'),
						'-',
						new Extension.Palette.Block('explode'),
						'-',
						new Extension.Palette.Block('explicitCommand'),
						new Extension.Palette.Block('fallibleCommand'),
						'-',
						new Extension.Palette.Block('fallibleReporter'),
						new Extension.Palette.Block('falliblePredicate'),
						'-',
						new Extension.Palette.Block('pickyboi'),
					],
					SpriteMorph
				),
				new Extension.PaletteCategory(
					'control',
					[
						new Extension.Palette.Block('receiveTestEvent'),
						new Extension.Palette.Block('printProcess'),
						'-',
						new Extension.Palette.Block('explode'),
						'-',
						new Extension.Palette.Block('explicitCommand'),
						new Extension.Palette.Block('fallibleCommand'),
						'-',
						new Extension.Palette.Block('fallibleReporter'),
						new Extension.Palette.Block('falliblePredicate'),
						'-',
						new Extension.Palette.Block('pickyboi'),
					],
					StageMorph
				),
				new Extension.PaletteCategory(
					'operators',
					[
						new Extension.Palette.Block('repeatString'),
						new Extension.Palette.Block('isEven'),
						new Extension.Palette.Block('addAll'),
						new Extension.Palette.Block('defaultAdder'),
					],
					SpriteMorph
				),
				new Extension.PaletteCategory(
					'operators',
					[
						new Extension.Palette.Block('repeatString'),
						new Extension.Palette.Block('isEven'),
						new Extension.Palette.Block('addAll'),
						new Extension.Palette.Block('defaultAdder'),
					],
					StageMorph
				),

            ];
        }

        getBlocks() {
            return [
				new Extension.Block(
					'logHelloWorld',
					'command',
					'Hello World',
					'Log Hello World!',
					[],
					function () { return window.ExampleExtension_fns.hello_world(); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'logHelloName',
					'command',
					'Hello World',
					'Log Hello %s',
					[],
					function (v0) { return window.ExampleExtension_fns.hello_name(v0); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'repeatString',
					'reporter',
					'operators',
					'Repeat %s for %times times',
					[],
					function (v0, v1) { return window.ExampleExtension_fns.repeat_text(v0, v1); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'isEven',
					'predicate',
					'operators',
					'is %num even?',
					[],
					function (v0) { return window.ExampleExtension_fns.is_even(v0); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'receiveTestEvent',
					'hat',
					'control',
					'on test event',
					[],
					function () { return window.ExampleExtension_fns.receive_test_event(); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'printProcess',
					'command',
					'control',
					'print process',
					[],
					function () { return window.ExampleExtension_fns.print_process(this, ); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'explode',
					'command',
					'control',
					'explode',
					[],
					function () { return window.ExampleExtension_fns.explode(); }
				).terminal().for(SpriteMorph, StageMorph),
				new Extension.Block(
					'addAll',
					'reporter',
					'operators',
					'add numbers %mult%num',
					[],
					function (v0) { return window.ExampleExtension_fns.add_all(v0.contents); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'explicitCommand',
					'command',
					'control',
					'explicit command',
					[],
					function () { return window.ExampleExtension_fns.explicit_command(); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'fallibleCommand',
					'command',
					'control',
					'fallible command',
					[],
					function () { return window.ExampleExtension_fns.fallible_command(); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'fallibleReporter',
					'reporter',
					'control',
					'fallible reporter',
					[],
					function () { return window.ExampleExtension_fns.fallible_reporter(); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'falliblePredicate',
					'predicate',
					'control',
					'fallible predicate',
					[],
					function () { return window.ExampleExtension_fns.fallible_predicate(); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'pickyboi',
					'reporter',
					'control',
					'picky boi %picky',
					[],
					function (v0) { return window.ExampleExtension_fns.picky_boi(v0); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'defaultAdder',
					'reporter',
					'operators',
					'add %n + %n',
					['7', '-4'],
					function (v0, v1) { return window.ExampleExtension_fns.default_adder(v0, v1); }
				).for(SpriteMorph, StageMorph),

            ];
        }

        getLabelParts() {
            return [
				new Extension.LabelPart(
					'times',
					() => {
						const part = new InputSlotMorph(
							null, // text
							true, // numeric
							null, // options
							false // readonly
						);
						return part;
					}
				),
				new Extension.LabelPart(
					'num',
					() => {
						const part = new InputSlotMorph(
							null, // text
							true, // numeric
							null, // options
							false // readonly
						);
						return part;
					}
				),
				new Extension.LabelPart(
					'picky',
					() => {
						const part = new InputSlotMorph(
							null, // text
							true, // numeric
							{"hello": "world","another": "option","nesting": {"deeper 1": {"deep 1": "deep val 1",},"deeper 2": {"deep 2": "deep val 2",},},"more stuff": {"thing": "some stuff","last one": "done",},}, // options
							true // readonly
						);
						return part;
					}
				),

            ];
        }

    }

    NetsBloxExtensions.register(ExampleExtension);
    let path = document.currentScript.src;
    path = path.substring(0, path.lastIndexOf("/"));
    var s = document.createElement('script');
    s.type = "module";
    s.innerHTML = `import init, {add_all, default_adder, explicit_command, explode, fallible_command, fallible_predicate, fallible_reporter, hello_name, hello_world, is_even, picky_boi, print_extension_name, print_hello_world, print_process, receive_test_event, repeat_text} from '${path}/pkg/netsblox_extension_rs.js';
    
    
        await init();

        window.ExampleExtension_fns = {};
		window.ExampleExtension_fns.add_all = add_all;
		window.ExampleExtension_fns.default_adder = default_adder;
		window.ExampleExtension_fns.explicit_command = explicit_command;
		window.ExampleExtension_fns.explode = explode;
		window.ExampleExtension_fns.fallible_command = fallible_command;
		window.ExampleExtension_fns.fallible_predicate = fallible_predicate;
		window.ExampleExtension_fns.fallible_reporter = fallible_reporter;
		window.ExampleExtension_fns.hello_name = hello_name;
		window.ExampleExtension_fns.hello_world = hello_world;
		window.ExampleExtension_fns.is_even = is_even;
		window.ExampleExtension_fns.picky_boi = picky_boi;
		window.ExampleExtension_fns.print_extension_name = print_extension_name;
		window.ExampleExtension_fns.print_hello_world = print_hello_world;
		window.ExampleExtension_fns.print_process = print_process;
		window.ExampleExtension_fns.receive_test_event = receive_test_event;
		window.ExampleExtension_fns.repeat_text = repeat_text;
        `;
    document.body.appendChild(s);
})();