int main(void) {
  int x = 1; 
  {
    int x = 2;
    return x;

    {
      int x = 1;
    }
  }
  return x;
}
