import { AnchorIdl } from '@codama/nodes-from-anchor';
import {
  assertIsNode,
  bottomUpTransformerVisitor,
  bytesTypeNode,
  bytesValueNode,
  DefinedTypeNode,
  fieldDiscriminatorNode,
  fixedSizeTypeNode,
  isNode,
  Node,
  NumberFormat,
  numberTypeNode,
  rootNodeVisitor,
  structFieldTypeNode,
  Visitor,
} from 'codama';

export const anchorTransformEventsToAccountsVisitor: (
  idl: AnchorIdl
) => Visitor<Node | null>[] = (idl) => {
  const eventsMap = new Map<string, number[]>();
  idl.events?.forEach((event) => {
    if ('discriminator' in event) {
      eventsMap.set(event.name.toLowerCase(), event.discriminator);
    }
  });

  return [
    rootNodeVisitor((node) => {
      const eventNodes: DefinedTypeNode[] = [];
      for (let i = 0; i < node.program.definedTypes.length; i++) {
        const definedTypeNode = node.program.definedTypes[i];
        if (
          isNode(definedTypeNode, 'definedTypeNode') &&
          eventsMap.has(definedTypeNode.name.toLowerCase())
        ) {
          eventNodes.push(...node.program.definedTypes.splice(i, 1));
          i--;
        }
      }

      for (const eventNode of eventNodes) {
        assertIsNode(eventNode.type, 'structTypeNode');

        const disc = eventsMap.get(eventNode.name.toLowerCase())!;
        node.program.accounts.push({
          kind: 'accountNode',
          name: eventNode.name,
          docs: [],
          data: {
            ...eventNode.type,
            fields: [
              structFieldTypeNode({
                name: 'discriminator',
                type: fixedSizeTypeNode(bytesTypeNode(), 8),
                defaultValue: bytesValueNode(
                  'base64',
                  Buffer.from(Uint8Array.from(disc)).toString('base64')
                ),
              }),
              ...eventNode.type.fields,
            ],
          },
          discriminators: [fieldDiscriminatorNode('discriminator')],
        });
      }

      return node;
    }) as Visitor<Node | null>,
  ];
};

export const jitoProgramsVisitor: (idl: AnchorIdl) => Visitor<Node | null>[] = (
  idl
) => {
  const changingTypeSet = new Set([
    'podU128',
    'podU64',
    'podU32',
    'podU16',
    'podBool',
  ]);
  return [
    bottomUpTransformerVisitor([
      {
        select: (nodes) => {
          const node = nodes[nodes.length - 1];
          return (
            isNode(node, 'structFieldTypeNode') &&
            isNode(node.type, 'definedTypeLinkNode') &&
            changingTypeSet.has(node.type.name)
          );
        },
        transform: (node, stack) => {
          assertIsNode(node, 'structFieldTypeNode');
          assertIsNode(node.type, 'definedTypeLinkNode');
          let retyped = node.type.name.toLowerCase().substring(3);
          if (retyped == 'bool') retyped = 'u8';

          return {
            ...node,
            type: numberTypeNode(retyped as NumberFormat),
          };
        },
      },
      {
        select: (nodes) => {
          const node = nodes[nodes.length - 1];
          return isNode(node, 'accountNode');
        },
        transform: (node) => {
          assertIsNode(node, 'accountNode');
          assertIsNode(node.data, 'structTypeNode');

          return {
            ...node,
            data: {
              ...node.data,
              fields: [
                structFieldTypeNode({
                  name: 'discriminator',
                  type: numberTypeNode('u64'),
                }),
                ...node.data.fields,
              ],
            },
          };
        },
      },
    ]),
  ];
};
