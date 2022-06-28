class Paginator {
    constructor(blockchainStrId, pairAddress, apiSandwichUrl, sectionHandle) {
        this.blockchainStrId = blockchainStrId;
        this.pairAddress = pairAddress;
        this.apiSandwichUrl = apiSandwichUrl;
        this.targetLoopDuration = 2000; // milliseconds
        this.sandwiches = [];
        this.blockRange = new Range();
        this.tokenMetadata;
        this.failed = false;
        this.errorMessage = "";
        this.sandwichesPerPage = 5;
        this.page = 0;
        this.sectionHandle = sectionHandle;

        this.scanData = document.createElement("p");
        this.scanData.className = "scan-data";
        this.sectionHandle.appendChild(this.scanData);
    }

    async runFetchLoop() {
        let before = -1;

        while (!this.failed && (before == -1 || before > 0)) {
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
                    console.log(`Alpha: Before: ${before}`);
                    this.updateScanData();
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

                console.log(`Beta: Before: ${before}`);

                // Sleep for a while and then try again.
                await this.padSleep(start);
                this.updateScanData();
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

            console.log(`Gamma: Before: ${before}`);

            // Sleep for a while and then try again.
            console.log(`Scan complete. ${data.sandwiches ? data.sandwiches.length : 0} sandwiches found. Retrying.`);
            this.updateScanData();
        }
    }

    updateScanData() {
        const blocksString = `Blocks Scanned: ${this.blockRange.totalScanned().toLocaleString()}`;
        const sandwichesFound = `Sandwiches Found: ${this.sandwiches.length.toLocaleString()}`;
        this.scanData.textContent = `${blocksString} / ${sandwichesFound}`;
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
        this.lowerBound;
        this.upperBound;
    }

    update(lowerBound, upperBound) {
        if (this.upperBound == null) {
            this.upperBound = upperBound;
        }

        this.lowerBound = lowerBound;
    }

    totalScanned() {
        if (this.lowerBound && this.upperBound) {
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