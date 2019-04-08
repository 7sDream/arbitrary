let visual = function () {

    let layout = {};
    let graph = {};

    let obj = {};
    let reds = [];
    let keepRed = false;

    function getGraphArg(max) {
        const ratio = layout.width / layout.height;
        const small = max + 1;
        const big = Math.ceil(max * (ratio > 1 ? ratio : 1 / ratio));
        let yRange = [-small, small];
        let xRange = [-big, big];
        if (ratio < 1) {
            [xRange, yRange] = [yRange, xRange];
        }
        return {
            xRange, yRange,
            width: layout.width, height: layout.height,
            pointSize: Math.max(Math.floor(layout.width / xRange[1] / 2), 1),
            dtick: Math.max(Math.floor(xRange[1] / 20), Math.floor(yRange[1] / 20)),
        }
    }

    function generateTrace(name, size, color, symbol) {
        return {
            x: [],
            y: [],
            mode: 'markers',
            type: 'scatter',
            name,
            text: [],
            marker: {
                size,
                color,
                symbol,
            },
        };
    }

    function generateInitLayout(arg) {
        Object.assign(layout, {
            title: "Visual Map",
            showlegend: false,
            clickmode: false,
            dragmode: false,
            hovermode: "closest",
            autosize: false,
            margin: 10,
            xaxis: {
                range: arg.xRange,
                dtick: arg.dtick,
            },
            yaxis: {
                range: arg.yRange,
                dtick: arg.dtick,
            }
        });
    }

    function generateInitData(pointSize) {
        const greenData = generateTrace("valid point", pointSize, "green", "square");
        const redData = generateTrace("invalid point", pointSize, "red", "x");
        const grayData = generateTrace("checking point", pointSize, "gray", "square");
        return [greenData, redData, grayData];
    }

    function countMessage(good, bad, waiting) {
        return `valid: ${good.length}, invalid: ${bad.length}, waiting check: ${waiting.length}`;
    }

    obj.init = function visual$init(eleId, max, kr) {
        keepRed = Boolean(kr);
        graph = document.getElementById(eleId)
        Plotly.react(
            eleId,
            [],
            layout,
            { displayModeBar: false },
        );
        const arg = getGraphArg(max);
        generateInitLayout(arg);
        Plotly.addTraces(graph, generateInitData(arg.pointSize));
        Plotly.relayout(graph, layout);
    };

    obj.update = function visual$update(data) {
        // delete waiting points
        graph.data[2].x.length = 0;
        graph.data[2].y.length = 0;
        if (!keepRed) {
            // delete bad points
            graph.data[1].x.length = 0;
            graph.data[1].y.length = 0;
        }

        // save all bads
        if (!keepRed) {
            reds.push(...data.bad);
        }

        // add new points
        Plotly.extendTraces(graph, {
            x: [
                data.good.map((point) => point[0]), // green
                data.bad.map((point) => point[0]), // red
                data.waiting.map((point) => point[0]), // gray
            ],
            y: [
                data.good.map((point) => point[1]), // gray
                data.bad.map((point) => point[1]), // red
                data.waiting.map((point) => point[1]), // gray
            ]
        }, [0, 1, 2]);

        // resize axis
        const newArg = getGraphArg(data.max);
        Plotly.relayout(graph, {
            title: 'Visual Map, Running, ' +
                countMessage(graph.data[0].x, !keepRed ? reds : graph.data[1].x, graph.data[2].x),
            'xaxis.range': newArg.xRange,
            'yaxis.range': newArg.yRange,
            'xaxis.dtick': newArg.dtick,
            'yaxis.dtick': newArg.dtick,
        });

        // resize marker 
        Plotly.restyle(graph, {
            'marker.size': newArg.pointSize,
        });

        if (graph.data[0].x.length > 1000) {
            Plotly.restyle(graph, {
                'visible': false,
            }, [0]);
        }
    };

    obj.finish = function visual$finish() {
        if (!keepRed) {
            graph.data[1].x.length = 0; graph.data[1].y.length = 0;
        }
        Plotly.deleteTraces(graph, [2]);
        Plotly.relayout(graph, {
            title: 'Visual Map, Drawing final result, please wait, ' +
                countMessage(graph.data[0].x, !keepRed ? reds : graph.data[1].x, []),
        });

        setTimeout(() => {
            if (!keepRed) {
                Plotly.extendTraces(graph, {
                    x: [
                        reds.map((point) => point[0]),
                    ],
                    y: [
                        reds.map((point) => point[1]),
                    ]
                }, [1]);
            }
            Plotly.restyle(graph, {
                'visible': true,
            }, [0]);
            Plotly.relayout(graph, {
                title: 'Visual Map, Finished, ' + countMessage(graph.data[0].x, !keepRed ? reds : graph.data[1].x, []),
            });
        }, 100);
    }

    return obj;
}();
