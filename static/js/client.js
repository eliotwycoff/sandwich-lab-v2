class Paginator {
    constructor(blockchainStrId, pairAddress, apiSandwichUrl, sectionHandle) {
        this.blockchainStrId = blockchainStrId;
        this.pairAddress = pairAddress;
        this.apiSandwichUrl = apiSandwichUrl;
        this.targetLoopDuration = 2000; // milliseconds
        this.fetching = false;
        this.sandwiches = [];
        this.blockRange = new Range();
        this.tokenMetadata;
        this.failed = false;
        this.errorMessage = "";
        this.sandwichesPerPage = 2;
        this.page = 0;
        this.sectionHandle = sectionHandle;

        // Create the scan metadata DOM elements.
        this.scanMetadata = create("div", ["scan-metadata"]);
        this.scanMetadataText = create("p", ["scan-metadata__text"]);
        this.scanMetadata.appendChild(this.scanMetadataText);
        this.sectionHandle.appendChild(this.scanMetadata);

        // Create the sandwich container DOM elements.
        this.sandwichContainers = [];
        //this.renderedSandwiches = [];

        for (let i = 0; i < this.sandwichesPerPage; i++) {
            this.sandwichContainers.push(create("div", ["sandwiches__container"]));
            this.sectionHandle.appendChild(this.sandwichContainers[i]);
        }

        // Create the pagination navigation bar.
        this.pageNav = create("div", ["page-nav", "row"]);
        this.sectionHandle.appendChild(this.pageNav);
    }

    async runFetchLoop() {
        if (this.fetching) return;
        this.fetching = true;

        let before = this.blockRange.lowerBound - 1;

        while (!this.failed && (before == -2 || before > 0) && !this.hasNextPage()) {
            // Keep track of how long each loop cycle takes.
            const start = new Date();

            console.log(`Range: ${this.blockRange.lowerBound} - ${this.blockRange.upperBound}`);

            // Fetch sandwich data from the sandwich api.
            const response = await fetch(this.getApiEndpoint(before));
            const data = await response.json();

            // If the scan failed, stop the loop and log the error.
            if (data.scan_metadata.failed) {
                this.failed = true;
                this.errorMessage = data.scan_metadata.error_message;
                break;
            }

            // Save the token metadata, if available and if necessary.
            if (data.token_metadata != null && this.tokenMetadata == null) {
                this.tokenMetadata = new TokenMetadata(
                    data.token_metadata.base_symbol,
                    data.token_metadata.quote_symbol,
                    data.token_metadata.native_symbol);
            }

            // If the scan hasn't completed:
            if (!data.scan_metadata.complete) {
                // If no sandwiches were found, sleep for a while
                // and then try again with the same `before` value.
                if (data.sandwiches == null) {
                    await this.padSleep(start);
                    this.updateDOM();
                    continue;
                }

                // Save the sandwiches that were found.
                for (let apiSandwich of data.sandwiches) {
                    this.sandwiches.push(this.parseSandwich(apiSandwich));
                }

                // Update the `blockRange`.
                this.blockRange.update(
                    data.fetch_metadata.lower_bound, 
                    data.fetch_metadata.upper_bound);

                // Update the `before` value.
                before = this.blockRange.lowerBound - 1;

                // Sleep for a while and then try again.
                await this.padSleep(start);
                this.updateDOM();
                continue;
            } 

            // Save any sandwiches that were found.
            if (data.sandwiches != null) {
                for (let apiSandwich of data.sandwiches) {
                    this.sandwiches.push(this.parseSandwich(apiSandwich));
                }
            }
            
            // Update the `blockRange`.
            this.blockRange.update(
                data.scan_metadata.lower_bound,
                data.scan_metadata.upper_bound);

            // Update the `before` value.
            before = this.blockRange.lowerBound - 1;

            // Sleep for a while and then try again.
            console.log(`Scan complete. ${data.sandwiches ? data.sandwiches.length : 0} sandwiches found. Retrying.`);
            this.updateDOM();
        }

        this.fetching = false;
    }

    // Change the DOM scan metadata and sandwich elements, if necessary,
    // and return a bool telling the scan loop to pause or not.
    updateDOM() {
        this.updateScanMetadata();

        // Get the range of sandwiches to render.
        const first = this.page * this.sandwichesPerPage; // inclusive
        let last = first + this.sandwichesPerPage; // not inclusive
        if (last > this.sandwiches.length) last = this.sandwiches.length;

        console.log(`First: ${first}`);
        console.log(`Last: ${last}`);

        for (let i = first; i < last; i++) {
            this.renderSandwich(i-first, this.sandwiches[i]);
        }

        for (let i = last; i < first + this.sandwichesPerPage; i++) {
            if (this.sandwichContainers[i-first]) {
                empty(this.sandwichContainers[i-first]);
            }
        }

        this.renderPageNav();
    }

    renderPageNav() {
        const prevButton = create("button", ["page-nav__prev-button", "page-nav__button", "button"], "Previous");
        const nextButton = create("button", ["page-nav__next-button", "page-nav__button", "button"], "  Next  ");

        prevButton.style.visibility = this.hasPrevPage() ? "visible" : "hidden";
        nextButton.style.visibility = this.hasNextPage() ? "visible" : "hidden";

        prevButton.addEventListener("click", () => {
            if (this.hasPrevPage()) {
                this.page -= 1;
                this.updateDOM();
            }
        });

        nextButton.addEventListener("click", () => {
            if (this.hasNextPage()) {
                this.page += 1;
                this.updateDOM();
                this.runFetchLoop();
            }
        });

        const statusContainer = create("div", ["page-nav__status"]);
        const statusSpan = create("span", ["page-nav__status__text"]);

        if (this.sandwiches.length > 0) {
            if (this.sandwichesPerPage == 1) {
                statusSpan.textContent = `Sandwich ${this.page + 1} of ${this.sandwiches.length}`;
            } else {
                const a = this.page * this.sandwichesPerPage + 1;
                let b = a + this.sandwichesPerPage - 1;
                b = b > this.sandwiches.length ? this.sandwiches.length : b;
                statusSpan.textContent = `Sandwiches ${a} to ${b} of ${this.sandwiches.length}`;
            }
        }
        
        statusContainer.appendChild(statusSpan);

        empty(this.pageNav);

        this.pageNav.appendChild(prevButton);
        this.pageNav.appendChild(statusContainer);
        this.pageNav.appendChild(nextButton);
    }

    hasPrevPage() {
        return this.page > 0;
    }

    hasNextPage() {
        return this.sandwiches.length > (this.page + 1) * this.sandwichesPerPage;
    }

    // Given an `index` to a DOM sandwich container, 
    // render the `sandwich` element to the DOM.
    renderSandwich(index, sandwich) {
        const sandwichElement = create("div", ["sandwiches__element", "card", "col"]);
        const sandwichElementLabel = create(
            "div", ["sandwiches__element__label", "card-label"], `Block ${sandwich.blockNumber.toLocaleString()}`);
        const sandwichElementBody = create("div", ["sandwiches__element__body", "card-body"]);
        const sandwichElementFooter = create("div", ["sandwiches__element__footer", "card-footer", "row"]);
        const sandwichElementFooterLeft = create("div", ["sandwiches__element__footer__left"]);
        const sandwichElementFooterCenter = create("div", ["sandwiches__element__footer__center"]);
        const sandwichElementFooterRight = create("div", ["sandwiches__element__footer__right"]);
        sandwichElementFooter.appendChild(sandwichElementFooterLeft);
        sandwichElementFooter.appendChild(sandwichElementFooterCenter);
        sandwichElementFooter.appendChild(sandwichElementFooterRight);

        sandwichElementBody.appendChild(this.renderSwap("Frontrun", sandwich.frontrun));

        for (let i = 0; i < sandwich.lunchmeat.length; i++) {
            sandwichElementBody.appendChild(this.renderSwap("Lunchmeat", sandwich.lunchmeat[i]));
        }

        sandwichElementBody.appendChild(this.renderSwap("Backrun", sandwich.backrun));

        const baseProfit = sandwich.attackerBaseProfit();
        const quoteProfit = sandwich.attackerQuoteProfit();
        const gas = sandwich.frontrun.gas + sandwich.backrun.gas;

        const baseProfitElement = create("span", ["revenue__base-profit"], 
            `${baseProfit < 0 ? "" : "+"}${baseProfit.toLocaleString()} ${this.tokenMetadata.baseSymbol}`);
        const profitConnector = create("span", ["revenue__connector"], " / ");
        const quoteProfitElement = create("span", ["revenue__quote-profit"],
            `${quoteProfit < 0 ? "" : "+"}${quoteProfit.toLocaleString()} ${this.tokenMetadata.quoteSymbol}`);
        const attackerProfitElement = create("p", ["revenue__attacker-profit", "revenue__text"]);
        const attackerGasElement = create("p", ["revenue__gas", "revenue__text", "negative"],
            `-${gas.toLocaleString()} ${this.tokenMetadata.nativeSymbol}`);
        const minerGasElement = create("p", ["revenue__gas", "revenue__text", "positive"],
            `+${gas.toLocaleString()} ${this.tokenMetadata.nativeSymbol}`);

        if (baseProfit > 0) baseProfitElement.classList.add("positive");
        if (quoteProfit > 0) quoteProfitElement.classList.add("positive");
        if (baseProfit < 0) baseProfitElement.classList.add("negative");
        if (quoteProfit < 0) quoteProfitElement.classList.add("negative");

        attackerProfitElement.appendChild(baseProfitElement);
        attackerProfitElement.appendChild(profitConnector);
        attackerProfitElement.appendChild(quoteProfitElement);
        
        sandwichElementFooterLeft.appendChild(create(
            "h3", ["revenue__title", "revenue__text"], `Attacker Profit & Loss`));
        sandwichElementFooterLeft.appendChild(attackerProfitElement);

        sandwichElementFooterCenter.appendChild(create(
            "h3", ["revenue__title", "revenue__text"], `Attacker Gas Fees`));
        sandwichElementFooterCenter.appendChild(attackerGasElement);

        sandwichElementFooterRight.appendChild(create(
            "h3", ["revenue__title", "revenue__text"], `Miner Profit`));
        sandwichElementFooterRight.appendChild(minerGasElement);

        sandwichElement.appendChild(sandwichElementLabel);
        sandwichElement.appendChild(sandwichElementBody);
        sandwichElement.appendChild(sandwichElementFooter);

        insertAsOnlyChild(this.sandwichContainers[index], sandwichElement);
    }

    renderSwap(label, swap) {
        const title = create("h3", ["swap__text", "swap__title"], `${label} Tx (Index ${swap.index})`);
        const hash = create("p", ["swap__text", "swap__hash"], `Tx Hash: ${swap.hash}`);

        const swapString = create("p", ["swap__text", "swap__io"]);
        swapString.appendChild(create("span", ["swap__io__label"], "Swap "));
        swapString.appendChild(swap.inputSpan(this.tokenMetadata));
        swapString.appendChild(create("span", ["swap__io__arrow"], " ➙ "));
        swapString.appendChild(swap.outputSpan(this.tokenMetadata));

        const swapElement = create("div", ["swap"]);
        swapElement.appendChild(title);
        swapElement.appendChild(hash);
        swapElement.appendChild(swapString);

        return swapElement;
    }

    updateScanMetadata() {
        const blocksString = `Blocks Scanned: ${this.blockRange.totalScanned().toLocaleString()}`;
        const sandwichesFound = `Sandwiches Found: ${this.sandwiches.length.toLocaleString()}`;
        this.scanMetadataText.textContent = `${blocksString} / ${sandwichesFound}`;
    }

    parseSandwich(apiSandwich) {
        return new Sandwich(
            apiSandwich.block_number,
            this.parseSwap(apiSandwich.frontrun),
            apiSandwich.lunchmeat.map(this.parseSwap),
            this.parseSwap(apiSandwich.backrun));
    }

    parseSwap(apiSwap) {
        return new Swap(
            apiSwap.hash,
            apiSwap.index,
            apiSwap.base_in,
            apiSwap.quote_in,
            apiSwap.base_out,
            apiSwap.quote_out,
            apiSwap.gas);
    }

    async padSleep(start) {
        const timeElapsed = new Date() - start;

        if (timeElapsed < this.targetLoopDuration) {
            await new Promise(r => setTimeout(r, this.targetLoopDuration - timeElapsed));
        }
    }

    getApiEndpoint(before) {
        const params = new URLSearchParams({
            blockchain: this.blockchainStrId,
            pair: this.pairAddress
        });

        if (before > 0) params.append("before", before);

        return `${this.apiSandwichUrl}?${params.toString()}`;
    }
}

class Range {
    constructor() {
        this.lowerBound = -1;
        this.upperBound = -1;
    }

    update(lowerBound, upperBound) {
        if (this.upperBound == -1) {
            this.upperBound = upperBound;
        }

        this.lowerBound = lowerBound;
    }

    totalScanned() {
        if (this.lowerBound != -1 && this.upperBound != -1) {
            return this.upperBound - this.lowerBound + 1;
        }

        return 0;
    }
}

class TokenMetadata {
    constructor(baseSymbol, quoteSymbol, nativeSymbol) {
        this.baseSymbol = baseSymbol;
        this.quoteSymbol = quoteSymbol;
        this.nativeSymbol = nativeSymbol;
    }
}

class Sandwich {
    constructor(blockNumber, frontrun, lunchmeat, backrun) {
        this.blockNumber = blockNumber;
        this.frontrun = frontrun;
        this.lunchmeat = lunchmeat;
        this.backrun = backrun;
    }

    attackerBaseProfit() {
        const longProfit = this.backrun.baseOut - this.frontrun.baseIn;
        const shortProfit = this.frontrun.baseOut - this.backrun.baseIn;
        return longProfit + shortProfit;
    }

    attackerQuoteProfit() {
        const longProfit = this.backrun.quoteOut - this.frontrun.quoteIn;
        const shortProfit = this.frontrun.quoteOut - this.backrun.quoteIn;
        return longProfit + shortProfit;
    }
}

class Swap {
    constructor(hash, index, baseIn, quoteIn, baseOut, quoteOut, gas) {
        this.hash = hash;
        this.index = index;
        this.baseIn = baseIn;
        this.quoteIn = quoteIn;
        this.baseOut = baseOut;
        this.quoteOut = quoteOut;
        this.gas = gas;
    }

    inputSpan(tokenMetadata) {
        return this.inputOutputSpan(tokenMetadata, this.baseIn, this.quoteIn);
    }

    outputSpan(tokenMetadata) {
        return this.inputOutputSpan(tokenMetadata, this.baseOut, this.quoteOut);
    }

    inputOutputSpan(tokenMetadata, baseAmount, quoteAmount) {
        let span = document.createElement("span");

        if (baseAmount > 0) {
            let baseSpan = create(
                "span", ["swap__io__base-text"], `${baseAmount.toLocaleString()} ${tokenMetadata.baseSymbol}`);
            span.appendChild(baseSpan);

            if (quoteAmount > 0) {
                let connector = create("span", ["swap__io__connector"], " & ");
                span.appendChild(connector);
            }
        }

        if (quoteAmount > 0) {
            let quoteSpan = create(
                "span", ["swap__io__quote-text"], `${quoteAmount.toLocaleString()} ${tokenMetadata.quoteSymbol}`);
            span.appendChild(quoteSpan);
        }
        
        return span;
    }
}

// This helper function applies the passed callback function 
// to the DOM element given by `elementId` after it's been rendered.
const setElement = (elementId, callback) => {
    const element = document.getElementById(elementId);

    if (element) {
        callback(element);
    } else {
        window.requestAnimationFrame(() => {
            setElement(elementId, callback);
        });
    }
};

// This helper function returns a handle to a newly created DOM element.
const create = (elementIdentifier, classList=[], textContent="") => {
    let element = document.createElement(elementIdentifier);
    element.classList.add(...classList);
    element.textContent = textContent;
    return element;
}

// This helper function removes all child nodes of `parent`.
const empty = (parent) => {
    while (parent.firstChild) {
        parent.removeChild(parent.firstChild);
    }
}

// This helper function replaces all child nodes of `parent` with `child`.
const insertAsOnlyChild = (parent, child) => {
    empty(parent);
    parent.appendChild(child);
};