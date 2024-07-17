/**
 * The following file is generated through a build script. Manually modifying it is an at-your-own-risk activity and your changes will likely be overridden.
 */

(function () {    
    class $NO_SPACE_EXTENSION_NAME extends Extension {
        constructor(ide) {
            super('$EXTENSION_NAME');
        }

        onOpenRole() {

        }

        getSettings() {
            return [
$SETTINGS
            ];
        }

        getMenu() {
            return {
$MENU
            };
        }

        getCategories() {
            return [
$CATEGORIES
            ];
        }

        getPalette() {
            return [
$PALETTE
            ];
        }

        getBlocks() {
            return [
$BLOCKS
            ];
        }

        getLabelParts() {
            return [
$LABELPARTS
            ];
        }

    }

    NetsBloxExtensions.register($NO_SPACE_EXTENSION_NAME);
    let path = document.currentScript.src;
    path = path.substring(0, path.lastIndexOf("/"));
    var s = document.createElement('script');
    s.type = "module";
    s.innerHTML = `import init, {$IMPORTS_LIST} from '${path}/pkg/$PACKAGE_NAME.js';

        await init();

        window.$NO_SPACE_EXTENSION_NAME_fns = {};
$WINDOW_IMPORTS

$SETUP_FUNC
        `;
    document.body.appendChild(s);
})();