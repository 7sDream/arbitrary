let solve = function () {

    const getDigitSum = function () {
        const cache = [];
        return (n) => {
            n = Math.abs(n);
            if (n < 10) {
                return n;
            } else {
                if (!cache[n]) {
                    cache[n] = ((n % 10) + getDigitSum((n / 10) >> 0));
                }
                return cache[n];
            }
        }
    }();

    function check(point, target) {
        const [x, y] = point;
        return getDigitSum(x) + getDigitSum(y) <= target;
    }

    function pointHash(point) {
        return point.toString();
    }

    function isInQS(point, qs) {
        return qs.has(pointHash(point));
    }

    function addToQs(point, qs) {
        qs.add(pointHash(point));
    }

    function addNeighbors(point, q, qs, max) {
        const [x, y] = point;
        let newMax = max;
        [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]].forEach((neighbor) => {
            // only need calculate 1/8 due to symmetry
            // other 7/8 will be calculated by extend function
            const [x, y] = neighbor;
            if (!isInQS(neighbor, qs) && x >= 0 && y >= 0 && x >= y) {
                q.push(neighbor);
                addToQs(neighbor, qs);
                newMax = Math.max(newMax, x, y);
            }
        });
        return newMax;
    }

    function oneLoopCount(target) {
        if (target <= 8) {
            return 1;
        } else if (target <= 12) {
            return 4 * (target - 8);
        } else if (target <= 17) {
            return 8 * (target - 10);
        } else {
            return 100;
        }
    }

    function extend(arr, container /* can be undefined, will use arr as container*/) {
        let length = arr.length;
        if (typeof(container) === 'undefined') {
            container = arr;
        }
        for (let i = 0; i < length; i++) {
            const [x, y] = arr[i];
            if (x === y && x === 0) {
                continue;
            }
            if (y == 0 || y == x) {
                container.push([-x, -y], [-y, x], [y, -x]);
            } else {
                container.push([x, -y], [-x, y], [-x, -y], [y, x], [y, -x], [-y, x], [-y, -x]);
            }
        }
    }

    return function* _solve(target, max) {
        // BFS queue
        const q = new Deque([[0, 0]]);

        // Already in queue point set
        const qs = new Set();
        addToQs([0, 0], qs);

        // Max value for point x and y
        let MAX = max;

        let MAX_COUNT = 1;

        while (true) {
            let count = 0, finish = true;
            const good = [], bad = [];
            while (!q.isEmpty()) {
                if (count === MAX_COUNT) {
                    finish = false;
                    break;
                }
                count += 1;
                const p = q.shift();
                if (check(p, target)) {
                    good.push(p);
                    MAX = addNeighbors(p, q, qs, MAX);
                } else {
                    bad.push(p);
                }
            }

            MAX_COUNT = q.length;

            extend(good);
            extend(bad);
            const waiting = q.toArray();
            extend(waiting);

            const updateData = {
                good, bad, waiting, 
                max: MAX,
            }

            if (finish) {
                return updateData;
            } else {
                yield updateData;
            }
        }
    }

}();
