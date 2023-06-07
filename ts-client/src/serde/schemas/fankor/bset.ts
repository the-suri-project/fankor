import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';
import { InferFnkBorshSchemaInner } from '../maps';
import { FnkBorshSchema } from '../../borsh';
import { U16, U8 } from '../unsigned';

export function FnkBSet<Sv extends FnkBorshSchema<any>>(valueSchema: Sv) {
    return new FnkBSetSchema(valueSchema);
}

export class FnkBSetSchema<Sv extends FnkBorshSchema<any>>
    implements FnkBorshSchema<InferFnkBorshSchemaInner<Sv>[]>
{
    readonly valueSchema: Sv;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(valueSchema: Sv) {
        this.valueSchema = valueSchema;
    }

    // METHODS ----------------------------------------------------------------

    /**
     * Values must be sorted.
     */
    serialize(writer: FnkBorshWriter, value: InferFnkBorshSchemaInner<Sv>[]) {
        let nodes: Node<InferFnkBorshSchemaInner<Sv>>[] = value.map((v) => {
            return {
                value: v,
                leftChildAt: 0,
                rightChildAt: 0,
                height: 0,
            };
        });

        let root = fixNode(0, nodes.length, nodes);

        if (!root) {
            U16.serialize(writer, 0);
            U16.serialize(writer, 0);
        } else {
            let [rootPosition, _] = root;

            U16.serialize(writer, nodes.length);
            U16.serialize(writer, rootPosition + 1);

            for (const node of nodes) {
                this.valueSchema.serialize(writer, node.value);
                U16.serialize(writer, node.leftChildAt);
                U16.serialize(writer, node.rightChildAt);
                U8.serialize(writer, node.height);
            }
        }
    }

    deserialize(reader: FnkBorshReader): InferFnkBorshSchemaInner<Sv>[] {
        const length = U16.deserialize(reader);
        U16.deserialize(reader);

        const result: InferFnkBorshSchemaInner<Sv>[] = [];

        for (let i = 0; i < length; i++) {
            const value = this.valueSchema.deserialize(reader);
            U16.deserialize(reader);
            U16.deserialize(reader);
            U8.deserialize(reader);

            result.push(value);
        }

        return result;
    }
}

interface Node<V> {
    value: V;
    leftChildAt: number;
    rightChildAt: number;
    height: number;
}

// [from, to)
function fixNode<V>(
    from: number,
    to: number,
    nodes: Node<V>[]
): [number, Node<V>] | null {
    if (from === to) {
        return null;
    }

    const centerPosition = Math.floor((from + to) / 2);
    const node = nodes[centerPosition];
    node.height = 0;

    const leftChild = fixNode(from, centerPosition, nodes);
    const rightChild = fixNode(centerPosition + 1, to, nodes);

    if (leftChild) {
        const [leftChildPosition, leftChildNode] = leftChild;
        node.leftChildAt = leftChildPosition + 1;
        node.height = 1 + leftChildNode.height;
    } else {
        node.leftChildAt = 0;
    }

    if (rightChild) {
        const [rightChildPosition, rightChildNode] = rightChild;
        node.rightChildAt = rightChildPosition + 1;
        node.height = Math.max(node.height, 1 + rightChildNode.height);
    } else {
        node.rightChildAt = 0;
    }

    return [centerPosition, node];
}
