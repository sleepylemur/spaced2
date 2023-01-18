import re
import os
import sys
from collections import defaultdict,deque
import datetime

class CARD_TYPES:
    BASIC = 'basic'
    CLOZE = 'cloze'
    
class SETTINGS:
    CARD_TYPE = CARD_TYPES.BASIC
    CARDS_PER_DAY = 20

def anki_print(x:str):
    print(f'anki> {x}')


def list_decks():
    decks = os.listdir(f'{os.getcwd()}/decks')
    if len(decks) == 0:
        print('No decks found.')
    else:
        print('Decks:')
        for deck in decks:
            print(f'  - {deck}')

def add_cards_to_deck():
    deck = input('Deck: ')
    try:
        lines = open(f'{os.getcwd()}/decks/{deck}', 'r').readlines()
        print(lines)
        if len(lines) == 0:
            current_id = 0
        else:
            current_id = int(lines[-3].split(':')[1].strip())
        print(current_id)
        while True:
            current_id += 1
            question = input('Question: ')
            answer = input('Answer: ')
            # if not os.path.exists(f'{os.getcwd()}/decks/{deck}'):
            with open(f'{os.getcwd()}/decks/{deck}', 'a') as f:

                f.write(f'id: {current_id}\n')
                f.write(f'question: {question}\n')
                f.write(f'answer: {answer}\n')
                f.write('\n')
            print('Card added.')
    except KeyboardInterrupt:
        print('')
        print('Exiting...')
        return

def get_card_scores(deck:str):
    history = open(f'{os.getcwd()}/history/{deck}', 'r').readlines()
    if len(history) == 0:
        return {}
    else:
        card_scores = {}
        for line in history:
            id, score,session_time = line.split('&')
            if id not in card_scores:
                card_scores[id] = {
                    'total': 1,
                    'correct': int(score),
                    "correct_streak": 1 if int(score) == 1 else 0,
                    "time":datetime.datetime.strptime(session_time.rstrip(), '%Y-%m-%d %H:%M:%S.%f'),
                    "card_due": datetime.datetime.strptime(session_time.rstrip(), '%Y-%m-%d %H:%M:%S.%f').date(),
                }
            else:
                card_scores[id]['total'] += 1
                card_scores[id]['correct'] += int(score)
                if int(score) == 1:
                    card_scores[id]['correct_streak'] += 1
                else:
                    card_scores[id]['correct_streak'] = 0
                if card_scores[id]['correct_streak'] > 2:
                    time_delta = datetime.datetime.strptime(session_time.rstrip(), '%Y-%m-%d %H:%M:%S.%f') - card_scores[id]['time']
                    if time_delta.days > 0:
                        card_scores[id]['card_due'] = card_scores[id]['card_due'] + datetime.timedelta(days=card_scores[id]['card_due'].days * 2)
                    else:
                        card_scores[id]['card_due'] = card_scores[id]['card_due'] + datetime.timedelta(days=1)
                else:
                    card_scores[id]['card_due'] = datetime.datetime.strptime(session_time.rstrip(), '%Y-%m-%d %H:%M:%S.%f').date()
                card_scores[id]['time'] = datetime.datetime.strptime(session_time.rstrip(), '%Y-%m-%d %H:%M:%S.%f')
                    
        return card_scores

def add_cards_to_review(active_card_stack, inactive_cards):
    print('Adding cards to review...')
    for _ in range(5):
        if len(inactive_cards) == 0:
            break
        active_card_stack.append(int(inactive_cards.pop()['id']) - 1)
    return active_card_stack

def train(active_card_stack, cards, history):
    try:
        while len(active_card_stack) != 0:
            card_index= active_card_stack.pop()
            card = cards[card_index]
            print(card['question'])
            answer = input('Your Answer: ')
            score = 1 if answer == card['answer'] else 0
            if answer == card['answer']:
                print('Correct!')
                card['correct_streak'] += 1
                if card['correct_streak'] < 3:
                    active_card_stack.appendleft(card_index)
            else:
                print(f'Incorrect\nAnswer: {card["answer"]}')
                card['correct_streak'] = 0
                active_card_stack.append(card_index)
            history.write(f'{card["id"]}&{score}&{datetime.datetime.now()}\n')
            print(len(active_card_stack))

        print('No more cards left.')
        history.close()
    except Exception as e:
        print(e)
        print('Exiting...')
        history.close()
    return

def get_card_data(deck):
    with open(f'{os.getcwd()}/decks/{deck}', 'r') as f:
        cards = get_cards(f.readlines())
    scores = get_card_scores(deck)
    for card in cards:
        if card['id'] in scores:
            card['card_due'] = scores[card['id']]['card_due']
    inactive_cards = [x for x in cards if x['id'] not in scores]
    active_cards = [x for x in cards if x['id'] in scores and scores[x['id']]['card_due'] <= datetime.datetime.now().date()]
    number_of_cards_to_review_today = len([x for x in scores.values() if x['card_due'] <= datetime.datetime.now().date()])
    history = open(f'{os.getcwd()}/history/{deck}', 'a')
    active_card_stack = deque([int(x['id']) - 1 for x in active_cards])
    active_cards.sort(key=lambda x: scores[x['id']]['correct_streak'] if x['id'] in scores else 0, reverse=True)
    for card in cards:
        if card['id'] in scores:
            card['correct_streak'] = scores[card['id']]['correct_streak']
        else:
            card['correct_streak'] = 0
    return cards, scores, inactive_cards, active_cards, number_of_cards_to_review_today, history, active_card_stack

def practice():
    decks = os.listdir(f'{os.getcwd()}/decks')
    for deck in decks[:5]:
        print(deck)
    if len(decks) == 0:
        print('No decks found.')
        return
    print('\n')
    while True:
        deck_path = input('Choose a deck: ')
        if not os.path.exists(f'{os.getcwd()}/decks/{deck_path}'):
            print('Deck not found.')
        else:
            break
    cards, scores, inactive_cards, active_cards, number_of_cards_to_review_today, history, active_card_stack = get_card_data(deck_path)
    print('total number of cards in the deck', len(cards))
    print('scores', scores)
    print('number of active cards', len(active_cards))
    print('number of inactive cards', len(inactive_cards))
    print('number of cards to review today', number_of_cards_to_review_today)
    print('active_card_stack', active_card_stack)
    print('active_cards', active_cards)

    while True:
        if number_of_cards_to_review_today == 0:
            if len(inactive_cards) == 0:
                print(f'Good job! No cards to review today.')
                break
            else:
                print(f'No cards to review today. Add cards? {len(inactive_cards)} available to add. (y/n)')
                command = input()
                if command == 'y':
                    active_card_stack = add_cards_to_review(active_card_stack, inactive_cards)
                    print('active_card_stack',active_card_stack)
                    # number_of_cards_to_review_today = len([x for x in scores.values() if x['card_due'] == datetime.datetime.now().date()])
                elif command == 'n':
                    break
                else:
                    print('Invalid command.')
        # print(f'{number_of_cards_to_review_today} cards to review today. Lets go!')
        train(active_card_stack, cards, history)
        cards, scores, inactive_cards, active_cards, number_of_cards_to_review_today, history, active_card_stack = get_card_data(deck_path)
    return

def get_cards(deck):
    cards = []
    for line in deck:
        if line.startswith('id:'):
            id = line.split(':')[1].strip()
        elif line.startswith('question:'):
            question = line.split(':')[1].strip()
        elif line.startswith('answer:'):
            answer = line.split(':')[1].strip()
            cards.append({
                'id': id,
                'question': question,
                'answer': answer
            })
    return cards

def version():
    print('Anki CLI v0.0.1')

def help():
    print('Available commands:')
    print('  - practice | p')
    print('  - add | a')
    print('  - list | l')
    print('  - edit | e')
    print('  - export | x')
    print('  - import | i')
    print('  - stats | s')
    print('  - version | v')
    print('  - help | h')
    print('  - quit | exit | q')
    print('')
    print('Type "help" for more information on a command.')
    print('')
    print('Type "help <command>" for more information on a specific command.')
    print('')
    print('Example: help practice')
    print('')
    print('Type "quit" to exit the program.')
    print('')

def main():
    with open('../banner.txt', 'r') as f:
        logo = f.read()
    print(logo)
    print('Welcome to the Anki CLI')
    print('-----------------------')
    print('')
    print('Available commands:')
    print('  - practice | p')
    print('  - add | a')
    print('  - list | l')
    print('  - edit | e')
    print('  - export | x')
    print('  - import | i')
    print('  - stats | s')
    print('  - version | v')
    print('  - help | h')
    print('  - quit | exit | q')
    print('')
    print('Type "help" for more information on a command.')
    print('')
    while True:
        command = input('anki> ')
        if command == 'quit' or command == 'exit' or command == 'q':
            sys.exit(0)
        elif command == 'help' or command == 'h':
            help()
        elif command == 'version' or command == 'v':
            version()
        elif command == 'add' or command == 'a':
            add_cards_to_deck()
        elif command == 'list' or command == 'l':
            list_decks()
        elif command == 'edit' or command == 'e':
            print('Edit is not yet implemented.')
        elif command == 'export' or command == 'x':
            print('Export is not yet implemented.')
        elif command == 'import' or command == 'i':
            print('Import is not yet implemented.')
        elif command == 'stats' or command == 's':
            print('Stats is not yet implemented.')
        elif command == 'practice' or command == 'p':
            practice()
        else:
            print('Command not recognized. Type "help" for a list of commands.')
    

if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        print('')
        print('Exiting...')
        sys.exit(0)