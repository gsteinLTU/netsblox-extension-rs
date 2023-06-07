(function () {    
    class ExampleExtension extends Extension {
        constructor(ide) {
            super('ExampleExtension');
        }

        onOpenRole() {

        }

        getSettings() {
            return [

            ];
        }

        getMenu() {
            return {
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
						new Extension.Palette.Block('logHelloName'),
						new Extension.Palette.Block('logHelloWorld'),
					],
					SpriteMorph
				),
				new Extension.PaletteCategory(
					'Hello World',
					[
						new Extension.Palette.Block('logHelloName'),
						new Extension.Palette.Block('logHelloWorld'),
					],
					StageMorph
				),
				new Extension.PaletteCategory(
					'operators',
					[
						new Extension.Palette.Block('repeatString'),
						new Extension.Palette.Block('isEven'),
					],
					SpriteMorph
				),
				new Extension.PaletteCategory(
					'operators',
					[
						new Extension.Palette.Block('repeatString'),
						new Extension.Palette.Block('isEven'),
					],
					StageMorph
				),

            ];
        }

        getBlocks() {
            return [
				new Extension.Block(
					'repeatString',
					'reporter',
					'operators',
					'Repeat %text for %times times',
					[],
					function (text, times) { return ExampleExtension_fns.repeat_text(text, times); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'isEven',
					'predicate',
					'operators',
					'is %num even?',
					[],
					function (num) { return ExampleExtension_fns.is_even(num); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'logHelloName',
					'command',
					'Hello World',
					'Log Hello %name',
					[],
					function (name) { return ExampleExtension_fns.hello_name(name); }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'logHelloWorld',
					'command',
					'Hello World',
					'Log Hello World!',
					[],
					function () { return ExampleExtension_fns.hello_world(); }
				).for(SpriteMorph, StageMorph),

            ];
        }

        getLabelParts() {
            return [
				new Extension.LabelPart(
					'%name',
					() => {
						const part = new InputSlotMorph(
							null, // text
							false, // is numeric
							null,
							false
						);
						return part;
					}
				),
				new Extension.LabelPart(
					'%num',
					() => {
						const part = new InputSlotMorph(
							null, // text
							true, // is numeric
							null,
							false
						);
						return part;
					}
				),
				new Extension.LabelPart(
					'%text',
					() => {
						const part = new InputSlotMorph(
							null, // text
							false, // is numeric
							null,
							false
						);
						return part;
					}
				),
				new Extension.LabelPart(
					'%times',
					() => {
						const part = new InputSlotMorph(
							null, // text
							true, // is numeric
							null,
							false
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
    s.innerHTML = `import init, {hello_world, hello_name, print_extension_name, is_even, repeat_text} from '${path}/pkg/netsblox_extension_rs.js';
    
    
        await init();

        window.ExampleExtension_fns = {};
		window.ExampleExtension_fns.hello_world = hello_world;
		window.ExampleExtension_fns.hello_name = hello_name;
		window.ExampleExtension_fns.print_extension_name = print_extension_name;
		window.ExampleExtension_fns.is_even = is_even;
		window.ExampleExtension_fns.repeat_text = repeat_text;
        `;
    document.body.appendChild(s);
})();